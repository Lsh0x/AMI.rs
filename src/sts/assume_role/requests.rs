//! Assume Role Request and Response Types

use crate::error::{AmiError, Result};
use crate::sts::Credentials;
use serde::{Deserialize, Serialize};

use super::model::AssumedRoleUser;

/// Request to assume an IAM role
///
/// # Example
///
/// ```rust
/// use wami::sts::AssumeRoleRequest;
///
/// let request = AssumeRoleRequest {
///     role_arn: "arn:aws:iam::123456789012:role/S3Access".to_string(),
///     role_session_name: "my-app-session".to_string(),
///     duration_seconds: Some(3600),
///     external_id: Some("unique-external-id".to_string()),
///     policy: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumeRoleRequest {
    /// The ARN of the role to assume
    pub role_arn: String,
    /// An identifier for the assumed role session
    pub role_session_name: String,
    /// The duration of the session in seconds (default: 3600, max: 43200)
    pub duration_seconds: Option<i32>,
    /// A unique identifier used by third parties for assuming a role
    pub external_id: Option<String>,
    /// An IAM policy in JSON format to further restrict permissions
    pub policy: Option<String>,
}

impl AssumeRoleRequest {
    /// Validate the request
    #[allow(clippy::result_large_err)]
    pub fn validate(&self) -> Result<()> {
        if self.role_arn.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Role ARN cannot be empty".to_string(),
            });
        }

        if self.role_session_name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Role session name cannot be empty".to_string(),
            });
        }

        // Validate session name format (alphanumeric, underscore, dash, plus, equals, comma, period, at sign, hyphen)
        if !self
            .role_session_name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '+' | '=' | ',' | '.' | '@'))
        {
            return Err(AmiError::InvalidParameter {
                message: "Role session name contains invalid characters".to_string(),
            });
        }

        // Validate duration if provided
        if let Some(duration) = self.duration_seconds {
            if !(900..=43200).contains(&duration) {
                return Err(AmiError::InvalidParameter {
                    message: "Duration must be between 900 and 43200 seconds".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Response from assuming a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumeRoleResponse {
    /// The temporary security credentials
    pub credentials: Credentials,
    /// Information about the assumed role user
    pub assumed_role_user: AssumedRoleUser,
}
