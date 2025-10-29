//! Session Token Request Types

use crate::error::{AmiError, Result};
use serde::{Deserialize, Serialize};

/// Request to get a session token
///
/// # Example
///
/// ```rust
/// use wami::sts::GetSessionTokenRequest;
///
/// let request = GetSessionTokenRequest {
///     duration_seconds: Some(3600),
///     serial_number: Some("arn:aws:iam::123456789012:mfa/alice".to_string()),
///     token_code: Some("123456".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionTokenRequest {
    /// The duration of the session in seconds
    pub duration_seconds: Option<i32>,
    /// The identification number of the MFA device
    pub serial_number: Option<String>,
    /// The value provided by the MFA device
    pub token_code: Option<String>,
}

impl GetSessionTokenRequest {
    /// Validate the request
    #[allow(clippy::result_large_err)]
    pub fn validate(&self) -> Result<()> {
        // Validate duration if provided
        if let Some(duration) = self.duration_seconds {
            if !(900..=129600).contains(&duration) {
                return Err(AmiError::InvalidParameter {
                    message: "Duration must be between 900 and 129600 seconds".to_string(),
                });
            }
        }

        // If serial number is provided, token code must also be provided
        if self.serial_number.is_some() && self.token_code.is_none() {
            return Err(AmiError::InvalidParameter {
                message: "Token code is required when serial number is provided".to_string(),
            });
        }

        // Validate token code format if provided
        if let Some(code) = &self.token_code {
            if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
                return Err(AmiError::InvalidParameter {
                    message: "Token code must be a 6-digit number".to_string(),
                });
            }
        }

        Ok(())
    }
}
