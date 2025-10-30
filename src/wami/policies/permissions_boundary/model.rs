//! Permissions Boundary Domain Model
//!
//! Represents permissions boundaries that set maximum permissions for users and roles.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a permissions boundary policy reference
///
/// A permissions boundary sets the maximum permissions that identity-based
/// policies can grant to a user or role. The effective permissions are the
/// intersection of identity-based policies and the permissions boundary.
///
/// # Example
///
/// ```rust
/// use wami::wami::policies::permissions_boundary::PermissionsBoundary;
/// use chrono::Utc;
///
/// let boundary = PermissionsBoundary {
///     policy_arn: "arn:aws:iam::123456789012:policy/boundary".to_string(),
///     attached_to: vec!["arn:aws:iam::123456789012:user/alice".to_string()],
///     created_date: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsBoundary {
    /// ARN of the policy being used as a boundary
    pub policy_arn: String,
    /// List of principals (users/roles) this boundary is attached to
    pub attached_to: Vec<String>,
    /// When this boundary configuration was created
    pub created_date: DateTime<Utc>,
}

impl PermissionsBoundary {
    /// Validate permissions boundary ARN format
    ///
    /// Ensures the ARN is valid and points to a managed policy.
    #[allow(clippy::result_large_err)]
    pub fn validate_arn(arn: &str) -> crate::error::Result<()> {
        if arn.is_empty() {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Permissions boundary ARN cannot be empty".to_string(),
            });
        }

        if !arn.starts_with("arn:") {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Permissions boundary must be a valid policy ARN".to_string(),
            });
        }

        if !arn.contains(":policy/") {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Permissions boundary must reference a managed policy".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_arn_valid() {
        assert!(
            PermissionsBoundary::validate_arn("arn:aws:iam::123456789012:policy/boundary").is_ok()
        );
        assert!(PermissionsBoundary::validate_arn(
            "arn:aws:iam::123456789012:policy/path/boundary"
        )
        .is_ok());
    }

    #[test]
    fn test_validate_arn_empty() {
        assert!(PermissionsBoundary::validate_arn("").is_err());
    }

    #[test]
    fn test_validate_arn_not_arn() {
        assert!(PermissionsBoundary::validate_arn("not-an-arn").is_err());
    }

    #[test]
    fn test_validate_arn_not_policy() {
        assert!(PermissionsBoundary::validate_arn("arn:aws:iam::123456789012:user/alice").is_err());
        assert!(PermissionsBoundary::validate_arn("arn:aws:iam::123456789012:role/admin").is_err());
    }

    #[test]
    fn test_permissions_boundary_creation() {
        let boundary = PermissionsBoundary {
            policy_arn: "arn:aws:iam::123456789012:policy/boundary".to_string(),
            attached_to: vec![
                "arn:aws:iam::123456789012:user/alice".to_string(),
                "arn:aws:iam::123456789012:role/admin".to_string(),
            ],
            created_date: Utc::now(),
        };

        assert_eq!(
            boundary.policy_arn,
            "arn:aws:iam::123456789012:policy/boundary"
        );
        assert_eq!(boundary.attached_to.len(), 2);
    }
}
