//! STS Assume Role Service
//!
//! Orchestrates role assumption operations.

use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::{AmiError, Result};
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
}

impl<S: SessionStore + RoleStore> AssumeRoleService<S> {
    /// Create a new AssumeRoleService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Assume an IAM role
    ///
    /// Returns temporary credentials for the assumed role.
    pub async fn assume_role(
        &self,
        context: &WamiContext,
        request: AssumeRoleRequest,
        principal_arn: &str,
    ) -> Result<AssumeRoleResponse> {
        // Validate request
        request.validate()?;

        // Verify role exists - try parsing as WAMI ARN first
        let role = if let Ok(wami_arn) = request.role_arn.parse::<crate::arn::WamiArn>() {
            if wami_arn.resource.resource_type == "role" {
                // Search for role by matching wami_arn
                let store_guard = self.store.read().unwrap();
                let (roles, _, _) = store_guard.list_roles(None, None).await?;
                roles
                    .into_iter()
                    .find(|r| r.wami_arn.to_string() == request.role_arn)
                    .ok_or_else(|| AmiError::ResourceNotFound {
                        resource: format!("Role: {}", request.role_arn),
                    })?
            } else {
                return Err(AmiError::InvalidParameter {
                    message: format!("ARN is not a role: {}", request.role_arn),
                });
            }
        } else {
            // Fall back to AWS format
            let role_name = self.extract_role_name_from_arn(&request.role_arn)?;
            self.store
                .read()
                .unwrap()
                .get_role(&role_name)
                .await?
                .ok_or_else(|| AmiError::ResourceNotFound {
                    resource: format!("Role: {}", role_name),
                })?
        };

        // Determine session duration (default: 1 hour, max: role's max session duration or 12 hours)
        let max_duration = role.max_session_duration.unwrap_or(43200);
        let duration_seconds = request.duration_seconds.unwrap_or(3600).min(max_duration);
        let expiration = Utc::now() + Duration::seconds(duration_seconds as i64);

        // Generate credentials
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
        let session_token = format!("TOKEN{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

        let session_arn = format!(
            "arn:aws:sts::{}:assumed-role/{}/{}",
            context.instance_id(),
            &role.role_name,
            request.role_session_name
        );

        // Build WAMI ARN for credentials using context
        let wami_arn = WamiArn::builder()
            .service(Service::Sts)
            .tenant_path(context.tenant_path().clone())
            .wami_instance(context.instance_id())
            .resource(
                "session",
                format!("{}/{}", role.role_name, request.role_session_name),
            )
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

        // Create assumed role user
        let assumed_role_id = format!(
            "AROA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );
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
        // Try parsing as WAMI ARN first
        if let Ok(wami_arn) = arn.parse::<crate::arn::WamiArn>() {
            if wami_arn.resource.resource_type == "role" {
                // For WAMI ARN, we need to get the role by its resource_id and look up the role_name
                // But since we only have resource_id, we'll need to search by it or modify the lookup
                // For now, we'll use the resource_id as identifier and search in store
                // The store.get_role() uses role_name, so we need a different approach
                // Actually, we should parse the WAMI ARN and extract resource_id, then search
                return Ok(wami_arn.resource.resource_id);
            }
        }

        // Fall back to AWS format: arn:aws:iam::123456789012:role/RoleName
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
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::identity::role::builder::build_role;

    fn setup_service() -> AssumeRoleService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        AssumeRoleService::new(store)
    }

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:iam:test:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("test"))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_assume_role() {
        let service = setup_service();
        let context = test_context();

        // Create a role
        let trust_policy = r#"{"Version":"2012-10-17","Statement":[]}"#;
        let role = build_role(
            "TestRole".to_string(),
            trust_policy.to_string(),
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let role_arn = role.wami_arn.to_string();

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
            .assume_role(&context, request, "arn:aws:iam::123456789012:user/alice")
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
            role_arn: "arn:wami:iam:test:wami:123456789012:role/nonexistent".to_string(),
            role_session_name: "test-session".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        let context = test_context();
        let result = service
            .assume_role(&context, request, "arn:aws:iam::123456789012:user/alice")
            .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(AmiError::ResourceNotFound { .. })));
    }

    #[tokio::test]
    async fn test_assume_role_with_external_id() {
        let service = setup_service();
        let context = test_context();

        // Create a role
        let trust_policy = r#"{"Version":"2012-10-17","Statement":[]}"#;
        let role = build_role(
            "CrossAccountRole".to_string(),
            trust_policy.to_string(),
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let role_arn = role.wami_arn.to_string();

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
            .assume_role(
                &context,
                request,
                "arn:aws:iam::999999999999:user/external-user",
            )
            .await
            .unwrap();

        assert!(response.credentials.expiration > Utc::now());
    }

    #[tokio::test]
    async fn test_assume_role_creates_session() {
        let service = setup_service();
        let context = test_context();

        // Create a role
        let trust_policy = r#"{"Version":"2012-10-17","Statement":[]}"#;
        let role = build_role(
            "SessionRole".to_string(),
            trust_policy.to_string(),
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let role_arn = role.wami_arn.to_string();

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
            .assume_role(&context, request, "arn:aws:iam::123456789012:user/bob")
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
