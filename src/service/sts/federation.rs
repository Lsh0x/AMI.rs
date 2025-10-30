//! STS Federation Service
//!
//! Orchestrates federated user token operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider, ResourceType};
use crate::store::traits::SessionStore;
use crate::wami::sts::federation::{
    FederatedUser, GetFederationTokenRequest, GetFederationTokenResponse,
};
use crate::wami::sts::session::SessionStatus;
use crate::wami::sts::{Credentials, StsSession};
use chrono::{Duration, Utc};
use std::sync::{Arc, RwLock};

/// Service for generating federated user tokens
///
/// Provides high-level operations for federation token creation.
pub struct FederationService<S> {
    store: Arc<RwLock<S>>,
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: SessionStore> FederationService<S> {
    /// Create a new FederationService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>, account_id: String) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
            account_id,
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
            account_id: self.account_id.clone(),
        }
    }

    /// Get a federation token
    ///
    /// Returns temporary credentials for a federated user.
    pub async fn get_federation_token(
        &self,
        request: GetFederationTokenRequest,
        principal_arn: &str,
    ) -> Result<GetFederationTokenResponse> {
        // Validate request
        request.validate()?;

        // Determine session duration (default: 12 hours, max: 36 hours)
        let duration_seconds = request.duration_seconds.unwrap_or(43200);
        let expiration = Utc::now() + Duration::seconds(duration_seconds as i64);

        // Generate credentials
        let access_key_id = self.provider.generate_resource_id(ResourceType::AccessKey);
        let secret_access_key = format!(
            "SECRET{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")
        );
        let session_token = format!("TOKEN{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

        let session_arn = format!(
            "arn:aws:sts::{}:federated-user/{}",
            self.account_id, request.name
        );
        let wami_arn = self.provider.generate_wami_arn(
            ResourceType::StsSession,
            &self.account_id,
            "/",
            &format!("federated-user/{}", request.name),
        );

        let credentials = Credentials {
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            session_token: session_token.clone(),
            expiration,
            arn: session_arn.clone(),
            wami_arn: wami_arn.clone(),
            providers: vec![],
            tenant_id: None,
        };

        // Create federated user
        let federated_user_id = self.provider.generate_resource_id(ResourceType::User);
        let federated_user = FederatedUser {
            federated_user_id,
            arn: session_arn.clone(),
        };

        // Create and store session
        let session = StsSession {
            session_token: session_token.clone(),
            access_key_id,
            secret_access_key,
            expiration,
            status: SessionStatus::Active,
            assumed_role_arn: None, // Federation doesn't assume a role
            federated_user_name: Some(request.name.clone()),
            principal_arn: Some(principal_arn.to_string()),
            arn: session_arn,
            wami_arn,
            providers: vec![],
            tenant_id: None,
            created_at: Utc::now(),
            last_used: None,
        };

        self.store.write().unwrap().create_session(session).await?;

        Ok(GetFederationTokenResponse {
            credentials,
            federated_user,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> FederationService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        FederationService::new(store, "123456789012".to_string())
    }

    #[tokio::test]
    async fn test_get_federation_token() {
        let service = setup_service();

        let request = GetFederationTokenRequest {
            name: "federated-user".to_string(),
            duration_seconds: Some(7200),
            policy: Some(r#"{"Version":"2012-10-17","Statement":[]}"#.to_string()),
        };

        let response = service
            .get_federation_token(request, "arn:aws:iam::123456789012:user/alice")
            .await
            .unwrap();

        assert!(!response.credentials.access_key_id.is_empty());
        assert!(!response.credentials.session_token.is_empty());
        assert!(response.federated_user.arn.contains("federated-user"));
        assert!(response.federated_user.arn.contains("federated-user"));
    }

    #[tokio::test]
    async fn test_get_federation_token_default_duration() {
        let service = setup_service();

        let request = GetFederationTokenRequest {
            name: "test-federated".to_string(),
            duration_seconds: None, // Should default to 12 hours
            policy: None,
        };

        let response = service
            .get_federation_token(request, "arn:aws:iam::123456789012:user/bob")
            .await
            .unwrap();

        assert!(response.credentials.expiration > Utc::now());
    }

    #[tokio::test]
    async fn test_get_federation_token_invalid_name() {
        let service = setup_service();

        let request = GetFederationTokenRequest {
            name: "invalid name with spaces".to_string(),
            duration_seconds: Some(3600),
            policy: None,
        };

        let result = service
            .get_federation_token(request, "arn:aws:iam::123456789012:user/alice")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_federation_token_creates_session() {
        let service = setup_service();

        let request = GetFederationTokenRequest {
            name: "session-check-user".to_string(),
            duration_seconds: Some(3600),
            policy: None,
        };

        let response = service
            .get_federation_token(request, "arn:aws:iam::123456789012:user/charlie")
            .await
            .unwrap();

        // Verify session was created
        let sessions = service
            .store
            .read()
            .unwrap()
            .list_sessions(None)
            .await
            .unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(
            sessions[0].session_token,
            response.credentials.session_token
        );
        assert!(sessions[0].assumed_role_arn.is_none()); // Federation doesn't assume roles
    }

    #[tokio::test]
    async fn test_get_federation_token_with_policy() {
        let service = setup_service();

        let policy = r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "s3:GetObject",
                "Resource": "*"
            }]
        }"#;

        let request = GetFederationTokenRequest {
            name: "s3-readonly-user".to_string(),
            duration_seconds: Some(7200),
            policy: Some(policy.to_string()),
        };

        let response = service
            .get_federation_token(request, "arn:aws:iam::123456789012:user/admin")
            .await
            .unwrap();

        assert!(!response.federated_user.federated_user_id.is_empty());
    }
}
