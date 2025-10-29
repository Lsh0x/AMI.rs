//! Session Token Domain Operations
//!
//! Pure business logic functions for session token operations.

use super::requests::*;
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::wami::sts::credentials;

/// Pure domain operations for session tokens
pub mod session_token_operations {
    use super::*;

    /// Generate session token credentials (pure function)
    pub fn generate_credentials(
        user_name: &str,
        duration_seconds: i32,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> credentials::Credentials {
        credentials::builder::build_credentials(
            format!("session:{}", user_name),
            duration_seconds,
            provider,
            account_id,
        )
    }

    /// Validate session token duration (pure function)
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

    /// Get default duration if not specified (pure function)
    pub fn get_default_duration() -> i32 {
        43200 // 12 hours
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_duration() {
        assert!(session_token_operations::validate_duration(3600).is_ok());
        assert!(session_token_operations::validate_duration(129600).is_ok());
        assert!(session_token_operations::validate_duration(100).is_err());
        assert!(session_token_operations::validate_duration(200000).is_err());
    }

    #[test]
    fn test_get_default_duration() {
        assert_eq!(session_token_operations::get_default_duration(), 43200);
    }
}
