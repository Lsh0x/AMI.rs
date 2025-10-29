//! Assume Role Domain Operations
//!
//! Pure business logic functions for assume role operations.

use super::{model::*, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::wami::sts::{credentials, session};

/// Pure domain operations for assume role
pub mod assume_role_operations {
    use super::*;

    /// Generate temporary credentials for assumed role (pure function)
    pub fn generate_credentials(
        role_name: &str,
        session_name: &str,
        duration_seconds: i32,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> credentials::Credentials {
        credentials::builder::build_credentials(
            format!("{}:{}", role_name, session_name),
            duration_seconds,
            provider,
            account_id,
        )
    }

    /// Validate role ARN format (pure function)
    pub fn validate_role_arn(role_arn: &str) -> Result<()> {
        if !role_arn.starts_with("arn:") {
            return Err(AmiError::InvalidParameter {
                message: format!("Invalid role ARN format: {}", role_arn),
            });
        }

        if !role_arn.contains(":role/") {
            return Err(AmiError::InvalidParameter {
                message: format!("ARN does not refer to a role: {}", role_arn),
            });
        }

        Ok(())
    }

    /// Validate session name format (pure function)
    pub fn validate_session_name(session_name: &str) -> Result<()> {
        if session_name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Session name cannot be empty".to_string(),
            });
        }

        if session_name.len() > 64 {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Session name '{}' exceeds maximum length of 64 characters",
                    session_name
                ),
            });
        }

        // Validate session name characters (alphanumeric, =,.@_-)
        if !session_name.chars().all(|c| {
            c.is_alphanumeric() || c == '=' || c == ',' || c == '.' || c == '@' || c == '_' || c == '-'
        }) {
            return Err(AmiError::InvalidParameter {
                message: format!("Session name '{}' contains invalid characters", session_name),
            });
        }

        Ok(())
    }

    /// Validate session duration (pure function)
    pub fn validate_duration(duration_seconds: i32, min: i32, max: i32) -> Result<()> {
        if duration_seconds < min || duration_seconds > max {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Duration {} seconds is outside valid range ({}-{} seconds)",
                    duration_seconds, min, max
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_role_arn() {
        assert!(assume_role_operations::validate_role_arn("arn:aws:iam::123456789012:role/MyRole").is_ok());
        assert!(assume_role_operations::validate_role_arn("invalid").is_err());
        assert!(assume_role_operations::validate_role_arn("arn:aws:iam::123:user/Alice").is_err());
    }

    #[test]
    fn test_validate_session_name() {
        assert!(assume_role_operations::validate_session_name("my-session").is_ok());
        assert!(assume_role_operations::validate_session_name("user@example.com").is_ok());
        assert!(assume_role_operations::validate_session_name("").is_err());
        assert!(assume_role_operations::validate_session_name("invalid space").is_err());
    }

    #[test]
    fn test_validate_duration() {
        assert!(assume_role_operations::validate_duration(3600, 900, 43200).is_ok());
        assert!(assume_role_operations::validate_duration(100, 900, 43200).is_err());
        assert!(assume_role_operations::validate_duration(50000, 900, 43200).is_err());
    }
}
