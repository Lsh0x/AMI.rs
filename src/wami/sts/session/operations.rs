//! Session Domain Operations
//!
//! Pure business logic functions for session management.

use super::model::*;
use crate::error::{AmiError, Result};
use crate::wami::sts::credentials;

/// Pure domain operations for sessions
pub mod session_operations {
    use super::*;

    /// Build a new session (pure function)
    pub fn build_session(
        session_name: String,
        role_arn: String,
        credentials: credentials::Credentials,
    ) -> StsSession {
        StsSession {
            session_name,
            assumed_role_arn: role_arn,
            credentials,
            create_date: chrono::Utc::now(),
        }
    }

    /// Check if session is expired (pure function)
    pub fn is_expired(session: &StsSession) -> bool {
        let now = chrono::Utc::now();
        session.expiration < now
    }

    /// Get remaining session time in seconds (pure function)
    pub fn remaining_seconds(session: &StsSession) -> i64 {
        let now = chrono::Utc::now();
        let remaining = session.expiration - now;
        remaining.num_seconds().max(0)
    }

    /// Validate session name format (pure function)
    pub fn validate_session_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Session name cannot be empty".to_string(),
            });
        }

        if name.len() > 64 {
            return Err(AmiError::InvalidParameter {
                message: format!("Session name '{}' exceeds maximum length of 64 characters", name),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_credentials(expire_in_seconds: i64) -> credentials::Credentials {
        credentials::Credentials {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            session_token: Some("token".to_string()),
            expiration: Utc::now() + Duration::seconds(expire_in_seconds),
            wami_arn: "arn:wami:sts::tenant:assumed-role/MyRole/session".to_string(),
            providers: vec![],
            tenant_id: None,
        }
    }

    #[test]
    fn test_build_session() {
        let creds = create_test_credentials(3600);
        let session = session_operations::build_session(
            "my-session".to_string(),
            "arn:aws:iam::123:role/MyRole".to_string(),
            creds,
        );

        assert_eq!(session.session_name, "my-session");
        assert_eq!(session.assumed_role_arn, "arn:aws:iam::123:role/MyRole");
    }

    #[test]
    fn test_is_expired() {
        let expired_creds = create_test_credentials(-100);
        let session = session_operations::build_session(
            "test".to_string(),
            "arn".to_string(),
            expired_creds,
        );
        assert!(session_operations::is_expired(&session));

        let valid_creds = create_test_credentials(3600);
        let session = session_operations::build_session(
            "test".to_string(),
            "arn".to_string(),
            valid_creds,
        );
        assert!(!session_operations::is_expired(&session));
    }

    #[test]
    fn test_remaining_seconds() {
        let creds = create_test_credentials(1800);
        let session = session_operations::build_session(
            "test".to_string(),
            "arn".to_string(),
            creds,
        );

        let remaining = session_operations::remaining_seconds(&session);
        assert!(remaining > 1700 && remaining <= 1800);
    }

    #[test]
    fn test_validate_session_name() {
        assert!(session_operations::validate_session_name("valid-session").is_ok());
        assert!(session_operations::validate_session_name("").is_err());
    }
}
