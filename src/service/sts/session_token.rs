//! STS Session Token Service
//!
//! Orchestrates session token generation operations.

use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::SessionStore;
use crate::wami::sts::session::SessionStatus;
use crate::wami::sts::session_token::GetSessionTokenRequest;
use crate::wami::sts::{Credentials, StsSession};
use chrono::{Duration, Utc};
use std::sync::{Arc, RwLock};

/// Response from getting a session token
#[derive(Debug, Clone)]
pub struct GetSessionTokenResponse {
    pub credentials: Credentials,
}

/// Service for generating session tokens
///
/// Provides high-level operations for session token creation.
pub struct SessionTokenService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: SessionStore> SessionTokenService<S> {
    /// Create a new SessionTokenService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Get a session token
    ///
    /// Generates temporary credentials for the current user.
    pub async fn get_session_token(
        &self,
        context: &WamiContext,
        request: GetSessionTokenRequest,
        principal_arn: &str,
    ) -> Result<GetSessionTokenResponse> {
        // Validate request
        request.validate()?;

        // Determine session duration (default: 1 hour, max: 36 hours)
        let duration_seconds = request.duration_seconds.unwrap_or(3600);
        let expiration = Utc::now() + Duration::seconds(duration_seconds as i64);

        // Generate credentials
        let session_token = format!("TOKEN{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        let access_key_id = format!(
            "AKIA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(16)
                .collect::<String>()
        );
        let secret_access_key = format!(
            "SECRET{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")
        );

        let session_arn = format!(
            "arn:aws:sts::{}:session/{}",
            context.instance_id(),
            &session_token[..16]
        );

        // Build WAMI ARN for credentials using context
        let wami_arn = WamiArn::builder()
            .service(Service::Sts)
            .tenant_path(context.tenant_path().clone())
            .wami_instance(context.instance_id())
            .resource("session", &session_token[..16])
            .build()?;

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

        // Create and store session
        let session = StsSession {
            session_token: session_token.clone(),
            access_key_id,
            secret_access_key,
            expiration,
            status: SessionStatus::Active,
            assumed_role_arn: None,
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

        Ok(GetSessionTokenResponse { credentials })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> SessionTokenService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        SessionTokenService::new(store)
    }

    fn test_context() -> crate::context::WamiContext {
        use crate::arn::{TenantPath, WamiArn};
        let arn: WamiArn = "arn:wami:.*:12345678:wami:123456789012:user/test"
            .parse()
            .unwrap();
        crate::context::WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_get_session_token() {
        let service = setup_service();
        let context = test_context();

        let request = GetSessionTokenRequest {
            duration_seconds: Some(3600),
            serial_number: None,
            token_code: None,
        };

        let response = service
            .get_session_token(&context, request, "arn:aws:iam::123456789012:user/alice")
            .await
            .unwrap();

        assert!(!response.credentials.access_key_id.is_empty());
        assert!(!response.credentials.session_token.is_empty());
        assert!(response.credentials.expiration > Utc::now());
    }

    #[tokio::test]
    async fn test_get_session_token_with_mfa() {
        let service = setup_service();
        let context = test_context();

        let request = GetSessionTokenRequest {
            duration_seconds: Some(7200),
            serial_number: Some("arn:aws:iam::123456789012:mfa/alice".to_string()),
            token_code: Some("123456".to_string()),
        };

        let response = service
            .get_session_token(&context, request, "arn:aws:iam::123456789012:user/alice")
            .await
            .unwrap();

        assert!(response.credentials.expiration > Utc::now());
    }

    #[tokio::test]
    async fn test_get_session_token_invalid_duration() {
        let service = setup_service();
        let context = test_context();

        let request = GetSessionTokenRequest {
            duration_seconds: Some(100), // Too short
            serial_number: None,
            token_code: None,
        };

        let result = service
            .get_session_token(&context, request, "arn:aws:iam::123456789012:user/alice")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_session_token_creates_session() {
        let service = setup_service();
        let context = test_context();

        let request = GetSessionTokenRequest {
            duration_seconds: Some(3600),
            serial_number: None,
            token_code: None,
        };

        let response = service
            .get_session_token(&context, request, "arn:aws:iam::123456789012:user/bob")
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
    }
}
