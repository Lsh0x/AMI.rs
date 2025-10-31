//! WAMI Context - Authentication and Authorization Context
//!
//! The `WamiContext` carries authentication and authorization information for all WAMI operations.
//! It is created during authentication and used throughout the system to determine:
//! - Which tenant and instance the operation targets
//! - Who is performing the operation (caller identity)
//! - Whether authorization checks should be applied
//!
//! # Security
//!
//! **CRITICAL:** Contexts should ONLY be created through `AuthenticationService.authenticate()`.
//! The builder is public for internal use and testing, but manually creating contexts
//! bypasses authentication and is a security risk.
//!
//! # Proper Usage Example
//!
//! ```rust,no_run
//! use wami::{AuthenticationService, store::memory::InMemoryWamiStore};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
//! let auth_service = AuthenticationService::new(store);
//!
//! // CORRECT: Authenticate first
//! let context = auth_service
//!     .authenticate("access_key_id", "secret_access_key")
//!     .await?;
//!
//! // Now use the authenticated context for operations
//! assert_eq!(context.instance_id(), "999888777");
//! # Ok(())
//! # }
//! ```

use crate::arn::{TenantPath, WamiArn};
use crate::error::{AmiError, Result};
use serde::{Deserialize, Serialize};

/// Session information for temporary credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session token identifier
    pub session_token: String,
    /// Session expiration time (Unix timestamp)
    pub expiration: i64,
    /// Assumed role ARN (if this is an assumed role session)
    pub assumed_role_arn: Option<WamiArn>,
}

/// WAMI Context - carries authentication and authorization information
///
/// This context is created during authentication and passed to all service operations.
/// It contains information about who is performing the operation and where it should be executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WamiContext {
    /// The tenant path where operations will be performed
    tenant_path: TenantPath,

    /// The WAMI instance ID
    instance_id: String,

    /// The ARN of the caller (user or assumed role)
    caller_arn: WamiArn,

    /// Whether the caller is a root user (bypasses all authorization)
    is_root: bool,

    /// Optional default region for operations
    region: Option<String>,

    /// Optional session information for temporary credentials
    session_info: Option<SessionInfo>,
}

impl WamiContext {
    /// Create a new context builder
    pub fn builder() -> WamiContextBuilder {
        WamiContextBuilder::default()
    }

    /// Check if the caller is a root user
    ///
    /// Root users have full access and bypass all authorization checks.
    pub fn is_root(&self) -> bool {
        self.is_root
    }

    /// Get the caller's ARN
    pub fn caller_arn(&self) -> &WamiArn {
        &self.caller_arn
    }

    /// Get the tenant path
    pub fn tenant_path(&self) -> &TenantPath {
        &self.tenant_path
    }

    /// Get the instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get the default region (if set)
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    /// Get session information (if temporary credentials)
    pub fn session_info(&self) -> Option<&SessionInfo> {
        self.session_info.as_ref()
    }

    /// Check if this context can access a specific tenant path
    ///
    /// A context can access:
    /// - Its own tenant
    /// - Any child tenant below it in the hierarchy
    /// - If root user: any tenant in the instance
    pub fn can_access_tenant(&self, target_tenant: &TenantPath) -> bool {
        // Root user can access any tenant
        if self.is_root {
            return true;
        }

        // Check if target tenant is the same or a child of context tenant
        target_tenant.starts_with(self.tenant_path())
    }

    /// Check if the session has expired (for temporary credentials)
    pub fn is_expired(&self) -> bool {
        if let Some(session) = &self.session_info {
            let now = chrono::Utc::now().timestamp();
            return now >= session.expiration;
        }
        false
    }
}

/// Builder for creating a WamiContext
#[derive(Default)]
pub struct WamiContextBuilder {
    tenant_path: Option<TenantPath>,
    instance_id: Option<String>,
    caller_arn: Option<WamiArn>,
    is_root: bool,
    region: Option<String>,
    session_info: Option<SessionInfo>,
}

impl WamiContextBuilder {
    /// Set the tenant path
    pub fn tenant_path(mut self, tenant_path: TenantPath) -> Self {
        self.tenant_path = Some(tenant_path);
        self
    }

    /// Set the instance ID
    pub fn instance_id(mut self, instance_id: impl Into<String>) -> Self {
        self.instance_id = Some(instance_id.into());
        self
    }

    /// Set the caller ARN
    pub fn caller_arn(mut self, caller_arn: WamiArn) -> Self {
        self.caller_arn = Some(caller_arn);
        self
    }

    /// Set whether the caller is a root user
    pub fn is_root(mut self, is_root: bool) -> Self {
        self.is_root = is_root;
        self
    }

    /// Set the default region
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set session information for temporary credentials
    pub fn session_info(mut self, session_info: SessionInfo) -> Self {
        self.session_info = Some(session_info);
        self
    }

    /// Build the WamiContext
    #[allow(clippy::result_large_err)]
    pub fn build(self) -> Result<WamiContext> {
        let tenant_path = self.tenant_path.ok_or_else(|| AmiError::InvalidParameter {
            message: "tenant_path is required".to_string(),
        })?;

        let instance_id = self.instance_id.ok_or_else(|| AmiError::InvalidParameter {
            message: "instance_id is required".to_string(),
        })?;

        let caller_arn = self.caller_arn.ok_or_else(|| AmiError::InvalidParameter {
            message: "caller_arn is required".to_string(),
        })?;

        // Validate that instance_id is not empty
        if instance_id.trim().is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "instance_id cannot be empty".to_string(),
            });
        }

        Ok(WamiContext {
            tenant_path,
            instance_id,
            caller_arn,
            is_root: self.is_root,
            region: self.region,
            session_info: self.session_info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let arn: WamiArn = "arn:wami:iam:12345678/87654321:wami:999888777:user/12345"
            .parse()
            .unwrap();

        let context = WamiContext::builder()
            .instance_id("999888777")
            .tenant_path(TenantPath::new(vec![12345678, 87654321]))
            .caller_arn(arn.clone())
            .is_root(false)
            .region("us-east-1")
            .build()
            .unwrap();

        assert_eq!(context.instance_id(), "999888777");
        assert_eq!(context.tenant_path().to_string(), "12345678/87654321");
        assert_eq!(context.caller_arn(), &arn);
        assert!(!context.is_root());
        assert_eq!(context.region(), Some("us-east-1"));
    }

    #[test]
    fn test_root_context() {
        let arn: WamiArn = "arn:wami:iam:0:wami:999888777:user/root".parse().unwrap();

        let context = WamiContext::builder()
            .instance_id("999888777")
            .tenant_path(TenantPath::single(0))
            .caller_arn(arn)
            .is_root(true)
            .build()
            .unwrap();

        assert!(context.is_root());
        assert_eq!(context.tenant_path().to_string(), "0");
    }

    #[test]
    fn test_can_access_tenant() {
        let arn: WamiArn = "arn:wami:iam:12345678:wami:999888777:user/12345"
            .parse()
            .unwrap();

        let context = WamiContext::builder()
            .instance_id("999888777")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap();

        // Can access same tenant
        assert!(context.can_access_tenant(&TenantPath::single(12345678)));

        // Can access child tenant
        assert!(context.can_access_tenant(&TenantPath::new(vec![12345678, 87654321])));

        // Cannot access sibling tenant
        assert!(!context.can_access_tenant(&TenantPath::single(99999999)));

        // Cannot access parent tenant (root)
        assert!(!context.can_access_tenant(&TenantPath::single(0)));
    }

    #[test]
    fn test_root_can_access_any_tenant() {
        let arn: WamiArn = "arn:wami:iam:0:wami:999888777:user/root".parse().unwrap();

        let context = WamiContext::builder()
            .instance_id("999888777")
            .tenant_path(TenantPath::single(0))
            .caller_arn(arn)
            .is_root(true)
            .build()
            .unwrap();

        // Root can access any tenant
        assert!(context.can_access_tenant(&TenantPath::single(0)));
        assert!(context.can_access_tenant(&TenantPath::single(12345678)));
        assert!(context.can_access_tenant(&TenantPath::new(vec![12345678, 87654321, 99999999])));
    }

    #[test]
    fn test_session_expiration() {
        let arn: WamiArn = "arn:wami:iam:12345678:wami:999888777:user/12345"
            .parse()
            .unwrap();

        let future_time = chrono::Utc::now().timestamp() + 3600; // 1 hour from now
        let session = SessionInfo {
            session_token: "token123".to_string(),
            expiration: future_time,
            assumed_role_arn: None,
        };

        let context = WamiContext::builder()
            .instance_id("999888777")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .session_info(session)
            .build()
            .unwrap();

        assert!(!context.is_expired());
    }

    #[test]
    fn test_expired_session() {
        let arn: WamiArn = "arn:wami:iam:12345678:wami:999888777:user/12345"
            .parse()
            .unwrap();

        let past_time = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
        let session = SessionInfo {
            session_token: "token123".to_string(),
            expiration: past_time,
            assumed_role_arn: None,
        };

        let context = WamiContext::builder()
            .instance_id("999888777")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .session_info(session)
            .build()
            .unwrap();

        assert!(context.is_expired());
    }

    #[test]
    fn test_context_builder_all_fields() {
        let arn: WamiArn = "arn:wami:iam:12345678:wami:999888777:user/12345"
            .parse()
            .unwrap();
        let future_time = chrono::Utc::now().timestamp() + 3600;
        let session = SessionInfo {
            session_token: "token123".to_string(),
            expiration: future_time,
            assumed_role_arn: None,
        };

        let context = WamiContext::builder()
            .instance_id("999888777")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn.clone())
            .is_root(false)
            .region("us-west-2")
            .session_info(session.clone())
            .build()
            .unwrap();

        assert_eq!(context.instance_id(), "999888777");
        assert_eq!(context.caller_arn(), &arn);
        assert_eq!(context.region(), Some("us-west-2"));
        assert_eq!(
            context.session_info().map(|s| s.session_token.as_str()),
            Some("token123")
        );
    }

    #[test]
    fn test_context_without_optional_fields() {
        let arn: WamiArn = "arn:wami:iam:12345678:wami:999888777:user/12345"
            .parse()
            .unwrap();

        let context = WamiContext::builder()
            .instance_id("999888777")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap();

        assert_eq!(context.region(), None);
        assert!(context.session_info().is_none());
    }

    #[test]
    fn test_missing_required_fields() {
        // Missing instance_id
        let result = WamiContext::builder()
            .tenant_path(TenantPath::single(0))
            .build();
        assert!(result.is_err());

        // Missing tenant_path
        let result = WamiContext::builder().instance_id("999888777").build();
        assert!(result.is_err());
    }
}
