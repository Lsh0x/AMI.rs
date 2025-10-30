//! STS Assume Role Service
//!
//! Orchestrates role assumption operations.

use crate::error::{AmiError, Result};
use crate::provider::{AwsProvider, CloudProvider, ResourceType};
use crate::store::traits::{RoleStore, SessionStore};
use crate::wami::sts::assume_role::{AssumeRoleRequest, AssumeRoleResponse, AssumedRoleUser};
use crate::wami::sts::session::SessionStatus;
use crate::wami::sts::{Credentials, StsSession};
use chrono::{Duration, Utc};
use std::sync::{Arc, RwLock};

/// Service for assuming IAM roles
///
/// Provides high-level operations for role assumption and temporary credentials.
pub struct AssumeRoleService<S> {
    store: Arc<RwLock<S>>,
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: SessionStore + RoleStore> AssumeRoleService<S> {
    /// Create a new AssumeRoleService with default AWS provider
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

    /// Assume an IAM role
    ///
    /// Returns temporary credentials for the assumed role.
    pub async fn assume_role(
        &self,
        request: AssumeRoleRequest,
        principal_arn: &str,
    ) -> Result<AssumeRoleResponse> {
        // Validate request
        request.validate()?;

        // Verify role exists
        let role_name = self.extract_role_name_from_arn(&request.role_arn)?;
        let role = self
            .store
            .read()
            .unwrap()
            .get_role(&role_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Role: {}", role_name),
            })?;

        // Determine session duration (default: 1 hour, max: role's max session duration or 12 hours)
        let max_duration = role.max_session_duration.unwrap_or(43200);
        let duration_seconds = request.duration_seconds.unwrap_or(3600).min(max_duration);
        let expiration = Utc::now() + Duration::seconds(duration_seconds as i64);

        // Generate credentials
        let access_key_id = self.provider.generate_resource_id(ResourceType::AccessKey);
        let secret_access_key = format!(
            "SECRET{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")
        );
        let session_token = format!("TOKEN{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

        let session_arn = format!(
            "arn:aws:sts::{}:assumed-role/{}/{}",
            self.account_id, role_name, request.role_session_name
        );
        let wami_arn = self.provider.generate_wami_arn(
            ResourceType::StsSession,
            &self.account_id,
            "/",
            &format!("{}/{}", role_name, request.role_session_name),
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

        // Create assumed role user
        let assumed_role_id = self.provider.generate_resource_id(ResourceType::Role);
        let assumed_role_user = AssumedRoleUser {
            assumed_role_id,
            arn: session_arn.clone(),
        };

        // Create and store session
        let session = StsSession {
            session_token: session_token.clone(),
            access_key_id,
            secret_access_key,
            expiration,
            status: SessionStatus::Active,
            assumed_role_arn: Some(request.role_arn.clone()),
            federated_user_name: None,
            principal_arn: Some(principal_arn.to_string()),
            arn: session_arn,
            wami_arn,
            providers: vec![],
            tenant_id: None,
            created_at: Utc::now(),
            last_used: None,
        };

        self.store.write().unwrap().create_session(session).await?;

        Ok(AssumeRoleResponse {
            credentials,
            assumed_role_user,
        })
    }

    // Helper methods

    fn extract_role_name_from_arn(&self, arn: &str) -> Result<String> {
        // Expected format: arn:aws:iam::123456789012:role/RoleName
        let parts: Vec<&str> = arn.split(':').collect();
        if parts.len() < 6 {
            return Err(AmiError::InvalidParameter {
                message: format!("Invalid role ARN: {}", arn),
            });
        }

        let resource_part = parts[5]; // "role/RoleName"
        let resource_parts: Vec<&str> = resource_part.split('/').collect();

        if resource_parts.len() < 2 {
            return Err(AmiError::InvalidParameter {
                message: format!("Invalid role ARN format: {}", arn),
            });
        }

        Ok(resource_parts[1..].join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::identity::role::builder::build_role;

    fn setup_service() -> AssumeRoleService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        AssumeRoleService::new(store, "123456789012".to_string())
    }

    #[tokio::test]
    async fn test_assume_role() {
        let service = setup_service();
        let provider = AwsProvider::new();

        // Create a role
        let trust_policy = r#"{"Version":"2012-10-17","Statement":[]}"#;
        let role = build_role(
            "TestRole".to_string(),
            trust_policy.to_string(),
            Some("/".to_string()),
            None,
            None,
            &provider,
            "123456789012",
        );

        let role_arn = role.arn.clone();

        service
            .store
            .write()
            .unwrap()
            .create_role(role)
            .await
            .unwrap();

        // Assume the role
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name: "test-session".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        let response = service
            .assume_role(request, "arn:aws:iam::123456789012:user/alice")
            .await
            .unwrap();

        assert!(!response.credentials.access_key_id.is_empty());
        assert!(!response.credentials.session_token.is_empty());
        assert!(response.assumed_role_user.arn.contains("assumed-role"));
        assert!(response.assumed_role_user.arn.contains("TestRole"));
    }

    #[tokio::test]
    async fn test_assume_role_nonexistent() {
        let service = setup_service();

        let request = AssumeRoleRequest {
            role_arn: "arn:aws:iam::123456789012:role/NonExistentRole".to_string(),
            role_session_name: "test-session".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        let result = service
            .assume_role(request, "arn:aws:iam::123456789012:user/alice")
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(AmiError::ResourceNotFound { .. })));
    }

    #[tokio::test]
    async fn test_assume_role_with_external_id() {
        let service = setup_service();
        let provider = AwsProvider::new();

        // Create a role
        let trust_policy = r#"{"Version":"2012-10-17","Statement":[]}"#;
        let role = build_role(
            "CrossAccountRole".to_string(),
            trust_policy.to_string(),
            Some("/".to_string()),
            None,
            None,
            &provider,
            "123456789012",
        );

        let role_arn = role.arn.clone();

        service
            .store
            .write()
            .unwrap()
            .create_role(role)
            .await
            .unwrap();

        // Assume with external ID
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name: "cross-account-session".to_string(),
            duration_seconds: Some(7200),
            external_id: Some("unique-external-id-12345".to_string()),
            policy: None,
        };

        let response = service
            .assume_role(request, "arn:aws:iam::999999999999:user/external-user")
            .await
            .unwrap();

        assert!(response.credentials.expiration > Utc::now());
    }

    #[tokio::test]
    async fn test_assume_role_creates_session() {
        let service = setup_service();
        let provider = AwsProvider::new();

        // Create a role
        let trust_policy = r#"{"Version":"2012-10-17","Statement":[]}"#;
        let role = build_role(
            "SessionRole".to_string(),
            trust_policy.to_string(),
            Some("/".to_string()),
            None,
            None,
            &provider,
            "123456789012",
        );

        let role_arn = role.arn.clone();

        service
            .store
            .write()
            .unwrap()
            .create_role(role)
            .await
            .unwrap();

        // Assume the role
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name: "check-session".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        let response = service
            .assume_role(request, "arn:aws:iam::123456789012:user/bob")
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
        assert!(sessions[0].assumed_role_arn.is_some());
    }

    #[tokio::test]
    async fn test_extract_role_name_from_arn() {
        let service = setup_service();

        let name = service
            .extract_role_name_from_arn("arn:aws:iam::123456789012:role/MyRole")
            .unwrap();
        assert_eq!(name, "MyRole");

        let name_with_path = service
            .extract_role_name_from_arn("arn:aws:iam::123456789012:role/path/to/MyRole")
            .unwrap();
        assert_eq!(name_with_path, "path/to/MyRole");
    }
}
