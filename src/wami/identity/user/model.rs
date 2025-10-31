//! User Domain Model
//!
//! Represents an IAM user entity

use crate::arn::WamiArn;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an IAM user
///
/// An IAM user is an entity that represents a person or service that interacts with cloud resources.
///
/// # Example
///
/// ```rust
/// use wami::wami::identity::user::User;
/// use wami::arn::{WamiArn, Service};
/// use chrono::Utc;
///
/// let wami_arn = WamiArn::builder()
///     .service(Service::Iam)
///     .tenant(12345678)
///     .wami_instance("main")
///     .resource("user", "AIDACKCEVSQ6C2EXAMPLE")
///     .build()
///     .unwrap();
///
/// let user = User {
///     user_name: "alice".to_string(),
///     user_id: "AIDACKCEVSQ6C2EXAMPLE".to_string(),
///     arn: "arn:aws:iam::123456789012:user/alice".to_string(),
///     path: "/".to_string(),
///     create_date: Utc::now(),
///     password_last_used: None,
///     permissions_boundary: None,
///     tags: vec![],
///     wami_arn,
///     providers: vec![],
///     tenant_id: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The friendly name identifying the user
    pub user_name: String,
    /// The stable and unique identifier for the user
    pub user_id: String,
    /// The Amazon Resource Name (ARN) that identifies the user
    pub arn: String,
    /// The path to the user
    pub path: String,
    /// The date and time when the user was created
    pub create_date: DateTime<Utc>,
    /// The date and time when the user's password was last used
    pub password_last_used: Option<DateTime<Utc>>,
    /// The ARN of the policy used to set the permissions boundary
    pub permissions_boundary: Option<String>,
    /// A list of tags associated with the user
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification (structured type)
    pub wami_arn: WamiArn,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
    /// Optional tenant ID for multi-tenant isolation
    pub tenant_id: Option<crate::wami::tenant::TenantId>,
}

impl User {
    /// Validate username format
    #[allow(clippy::result_large_err)]
    pub fn validate_username(name: &str) -> crate::error::Result<()> {
        if name.is_empty() {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Username cannot be empty".to_string(),
            });
        }
        if name.len() > 64 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Username exceeds 64 characters".to_string(),
            });
        }
        // Check for invalid characters (alphanumeric, underscore, dash, plus, equals, comma, period, at sign, hyphen)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '+' | '=' | ',' | '.' | '@'))
        {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Username contains invalid characters".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(User::validate_username("alice").is_ok());
        assert!(User::validate_username("user123").is_ok());
        assert!(User::validate_username("user-name").is_ok());
        assert!(User::validate_username("user_name").is_ok());
        assert!(User::validate_username("user+name").is_ok());
        assert!(User::validate_username("user=name").is_ok());
        assert!(User::validate_username("user,name").is_ok());
        assert!(User::validate_username("user.name").is_ok());
        assert!(User::validate_username("user@name").is_ok());
    }

    #[test]
    fn test_validate_username_empty() {
        let result = User::validate_username("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::AmiError::InvalidParameter { .. }
        ));
    }

    #[test]
    fn test_validate_username_too_long() {
        let long_name = "a".repeat(65);
        let result = User::validate_username(&long_name);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::AmiError::InvalidParameter { .. }
        ));
    }

    #[test]
    fn test_validate_username_invalid_chars() {
        let result = User::validate_username("user name");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::AmiError::InvalidParameter { .. }
        ));
    }
}
