//! Root User - Special admin user with full access per WAMI instance
//!
//! The root user is a special user that exists for each WAMI instance. It has:
//! - Full access to all resources in the instance (bypasses all authorization checks)
//! - Special ARN format: `arn:wami:iam:root:wami:{instance_id}:user/root`
//! - Cannot be deleted
//! - Created automatically during instance initialization
//!
//! The root user is similar to the AWS root account user and should be used:
//! - During initial instance setup
//! - For creating the first administrative users
//! - For emergency access when other accounts are locked
//! - Sparingly in production (prefer delegated admin users with specific permissions)
//!
//! # Security Recommendations
//!
//! 1. **Rotate root credentials regularly** - Change access keys periodically
//! 2. **Enable MFA** - Add multi-factor authentication for root user
//! 3. **Limit usage** - Use root only for tasks that require it
//! 4. **Monitor access** - Log and audit all root user operations
//! 5. **Delegate permissions** - Create admin users for day-to-day operations
//!
//! # Example
//!
//! ```rust
//! use wami::RootUser;
//!
//! // Create a root user for an instance
//! let root_user = RootUser::for_instance("999888777");
//!
//! assert_eq!(root_user.user_name(), "root");
//! assert!(root_user.is_root());
//! assert_eq!(
//!     root_user.arn().to_string(),
//!     "arn:wami:iam:root:wami:999888777:user/root"
//! );
//! ```

use crate::arn::{Service, TenantPath, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::wami::identity::User;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Constants for root user
pub const ROOT_USER_NAME: &str = "root";
pub const ROOT_TENANT: &str = "root";
pub const ROOT_USER_ID: &str = "root";

/// Root User - Special administrative user with full access
///
/// The root user is automatically created for each WAMI instance and has
/// unrestricted access to all resources within that instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootUser {
    /// The underlying User object
    user: User,
}

impl RootUser {
    /// Create a root user for a WAMI instance
    ///
    /// # Arguments
    ///
    /// * `instance_id` - The unique identifier for the WAMI instance
    ///
    /// # Example
    ///
    /// ```
    /// use wami::RootUser;
    ///
    /// let root_user = RootUser::for_instance("999888777");
    /// assert_eq!(root_user.user_name(), "root");
    /// ```
    pub fn for_instance(instance_id: impl Into<String>) -> Self {
        let instance_id = instance_id.into();
        let now = Utc::now();

        // Build root user ARN: arn:wami:iam:root:wami:{instance_id}:user/root
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant_path(TenantPath::single(ROOT_TENANT))
            .wami_instance(instance_id.clone())
            .resource("user", ROOT_USER_ID)
            .build()
            .expect("Root user ARN should always be valid");

        let user = User {
            user_name: ROOT_USER_NAME.to_string(),
            user_id: ROOT_USER_ID.to_string(),
            wami_arn: arn,
            arn: format!("arn:aws:iam::{instance_id}:user/root"), // AWS-compatible ARN
            path: "/".to_string(),
            create_date: now,
            password_last_used: None,
            permissions_boundary: None,
            tags: vec![],
            providers: vec![],
            tenant_id: None,
        };

        Self { user }
    }

    /// Check if this is the root user
    ///
    /// Always returns `true` for `RootUser` instances.
    pub fn is_root(&self) -> bool {
        true
    }

    /// Get the user name (always "root")
    pub fn user_name(&self) -> &str {
        &self.user.user_name
    }

    /// Get the user ID (always "root")
    pub fn user_id(&self) -> &str {
        &self.user.user_id
    }

    /// Get the WAMI ARN
    pub fn arn(&self) -> &WamiArn {
        &self.user.wami_arn
    }

    /// Get the AWS-compatible ARN
    pub fn aws_arn(&self) -> &str {
        &self.user.arn
    }

    /// Get the instance ID from the ARN
    pub fn instance_id(&self) -> &str {
        &self.user.wami_arn.wami_instance_id
    }

    /// Get the underlying User object
    pub fn as_user(&self) -> &User {
        &self.user
    }

    /// Convert into the underlying User object
    pub fn into_user(self) -> User {
        self.user
    }

    /// Create a WamiContext for this root user (internal use only)
    ///
    /// **SECURITY:** This should only be called internally after authentication.
    /// External callers must authenticate via `AuthenticationService`.
    ///
    /// This is kept for internal testing and migration purposes, but should not
    /// be exposed in the public API without authentication.
    #[doc(hidden)]
    #[allow(dead_code, clippy::result_large_err)] // Used in tests and may be needed for future internal operations
    pub(crate) fn create_context_internal(&self) -> Result<WamiContext> {
        WamiContext::builder()
            .instance_id(self.instance_id())
            .tenant_path(TenantPath::single(ROOT_TENANT))
            .caller_arn(self.user.wami_arn.clone())
            .is_root(true)
            .build()
    }

    /// Create a WamiContext for this root user with a specific region (internal use only)
    ///
    /// **SECURITY:** This should only be called internally after authentication.
    #[doc(hidden)]
    #[allow(dead_code, clippy::result_large_err)] // Used in tests and may be needed for future internal operations
    pub(crate) fn create_context_with_region_internal(
        &self,
        region: impl Into<String>,
    ) -> Result<WamiContext> {
        WamiContext::builder()
            .instance_id(self.instance_id())
            .tenant_path(TenantPath::single(ROOT_TENANT))
            .caller_arn(self.user.wami_arn.clone())
            .is_root(true)
            .region(region)
            .build()
    }
}

impl From<RootUser> for User {
    fn from(root_user: RootUser) -> Self {
        root_user.user
    }
}

impl AsRef<User> for RootUser {
    fn as_ref(&self) -> &User {
        &self.user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_root_user() {
        let root_user = RootUser::for_instance("999888777");

        assert_eq!(root_user.user_name(), "root");
        assert_eq!(root_user.user_id(), "root");
        assert!(root_user.is_root());
        assert_eq!(root_user.instance_id(), "999888777");
    }

    #[test]
    fn test_root_user_arn() {
        let root_user = RootUser::for_instance("999888777");
        let arn = root_user.arn();

        assert_eq!(arn.service, Service::Iam);
        assert_eq!(arn.tenant_path.as_string(), "root");
        assert_eq!(arn.wami_instance_id, "999888777");
        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:root:wami:999888777:user/root"
        );
    }

    #[test]
    fn test_root_user_aws_arn() {
        let root_user = RootUser::for_instance("999888777");

        assert_eq!(root_user.aws_arn(), "arn:aws:iam::999888777:user/root");
    }

    #[test]
    fn test_create_context_internal() {
        let root_user = RootUser::for_instance("999888777");
        let context = root_user.create_context_internal().unwrap();

        assert!(context.is_root());
        assert_eq!(context.instance_id(), "999888777");
        assert_eq!(context.tenant_path().to_string(), "root");
        assert_eq!(
            context.caller_arn().to_string(),
            root_user.arn().to_string()
        );
    }

    #[test]
    fn test_create_context_with_region_internal() {
        let root_user = RootUser::for_instance("999888777");
        let context = root_user
            .create_context_with_region_internal("us-east-1")
            .unwrap();

        assert!(context.is_root());
        assert_eq!(context.region(), Some("us-east-1"));
    }

    #[test]
    fn test_convert_to_user() {
        let root_user = RootUser::for_instance("999888777");
        let user: User = root_user.into_user();

        assert_eq!(user.user_name, "root");
        assert_eq!(user.user_id, "root");
    }

    #[test]
    fn test_as_ref_user() {
        let root_user = RootUser::for_instance("999888777");
        let user_ref: &User = root_user.as_ref();

        assert_eq!(user_ref.user_name, "root");
    }
}
