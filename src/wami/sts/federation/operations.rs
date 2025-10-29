//! Federation Domain Operations
//!
//! Pure business logic functions for federation token operations.

use super::{model::*, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::wami::sts::credentials;

/// Pure domain operations for federation
pub mod federation_operations {
    use super::*;

    /// Generate federation token credentials (pure function)
    pub fn generate_credentials(
        name: &str,
        duration_seconds: i32,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> credentials::Credentials {
        credentials::builder::build_credentials(
            format!("federation:{}", name),
            duration_seconds,
            provider,
            account_id,
        )
    }

    /// Validate federation user name (pure function)
    pub fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Name cannot be empty".to_string(),
            });
        }

        if name.len() > 32 {
            return Err(AmiError::InvalidParameter {
                message: format!("Name '{}' exceeds maximum length of 32 characters", name),
            });
        }

        // Validate name characters
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(AmiError::InvalidParameter {
                message: format!("Name '{}' contains invalid characters", name),
            });
        }

        Ok(())
    }

    /// Validate federation duration (pure function)
    pub fn validate_duration(duration_seconds: i32) -> Result<()> {
        const MIN_DURATION: i32 = 900; // 15 minutes
        const MAX_DURATION: i32 = 129600; // 36 hours

        if duration_seconds < MIN_DURATION || duration_seconds > MAX_DURATION {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Duration {} seconds is outside valid range ({}-{} seconds)",
                    duration_seconds, MIN_DURATION, MAX_DURATION
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
    fn test_validate_name() {
        assert!(federation_operations::validate_name("bob").is_ok());
        assert!(federation_operations::validate_name("user-123").is_ok());
        assert!(federation_operations::validate_name("").is_err());
        assert!(federation_operations::validate_name("invalid space").is_err());
    }

    #[test]
    fn test_validate_duration() {
        assert!(federation_operations::validate_duration(3600).is_ok());
        assert!(federation_operations::validate_duration(129600).is_ok());
        assert!(federation_operations::validate_duration(100).is_err());
        assert!(federation_operations::validate_duration(200000).is_err());
    }
}
