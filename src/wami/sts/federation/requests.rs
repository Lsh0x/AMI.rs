//! Federation Request and Response Types

use crate::error::{AmiError, Result};
use crate::wami::sts::Credentials;
use serde::{Deserialize, Serialize};

use super::model::FederatedUser;

/// Request to get a federation token
///
/// # Example
///
/// ```rust
/// use wami::wami::sts::federation::GetFederationTokenRequest;
///
/// let request = GetFederationTokenRequest {
///     name: "federated-user".to_string(),
///     duration_seconds: Some(3600),
///     policy: Some(r#"{"Version":"2012-10-17","Statement":[]}"#.to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFederationTokenRequest {
    /// The name of the federated user
    pub name: String,
    /// The duration of the session in seconds
    pub duration_seconds: Option<i32>,
    /// An IAM policy in JSON format
    pub policy: Option<String>,
}

impl GetFederationTokenRequest {
    /// Validate the request
    #[allow(clippy::result_large_err)]
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Name cannot be empty".to_string(),
            });
        }

        // Validate name format
        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '@'))
        {
            return Err(AmiError::InvalidParameter {
                message: "Name contains invalid characters".to_string(),
            });
        }

        // Validate duration if provided
        if let Some(duration) = self.duration_seconds {
            if !(900..=129600).contains(&duration) {
                return Err(AmiError::InvalidParameter {
                    message: "Duration must be between 900 and 129600 seconds".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Response from getting a federation token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFederationTokenResponse {
    /// The temporary security credentials
    pub credentials: Credentials,
    /// Information about the federated user
    pub federated_user: FederatedUser,
}
