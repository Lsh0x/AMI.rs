//! Session Domain Model

use crate::arn::WamiArn;
use serde::{Deserialize, Serialize};

/// Represents an STS session with temporary credentials
///
/// # Example
///
/// ```rust
/// use wami::wami::sts::session::{StsSession, SessionStatus};
/// use chrono::Utc;
///
/// let session = StsSession {
///     session_token: "FwoGZXIvYXdzEBYaDH...".to_string(),
///     access_key_id: "ASIAIOSFODNN7EXAMPLE".to_string(),
///     secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
///     expiration: Utc::now() + chrono::Duration::hours(1),
///     status: SessionStatus::Active,
///     assumed_role_arn: Some("arn:aws:iam::123456789012:role/MyRole".to_string()),
///     federated_user_name: None,
///     principal_arn: None,
///     arn: "arn:aws:sts::123456789012:assumed-role/MyRole/session-name".to_string(),
///     wami_arn: "arn:wami:sts:root:wami:123456789012:session/session-id".parse().unwrap(),
///     providers: vec![],
///     tenant_id: None,
///     created_at: Utc::now(),
///     last_used: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StsSession {
    /// The session token for temporary credentials
    pub session_token: String,
    /// The access key ID
    pub access_key_id: String,
    /// The secret access key
    pub secret_access_key: String,
    /// When the credentials expire
    pub expiration: chrono::DateTime<chrono::Utc>,
    /// Current status of the session
    pub status: SessionStatus,
    /// The ARN of the assumed role (if any)
    pub assumed_role_arn: Option<String>,
    /// The name of the federated user (if any)
    pub federated_user_name: Option<String>,
    /// The ARN of the principal (if any)
    pub principal_arn: Option<String>,
    /// The native cloud provider ARN (e.g., AWS assumed-role ARN)
    pub arn: String,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: WamiArn,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
    /// Optional tenant ID for multi-tenant isolation
    pub tenant_id: Option<crate::wami::tenant::TenantId>,
    /// When the session was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the session was last used
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Status of an STS session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is active and valid
    Active,
    /// Session has expired
    Expired,
    /// Session was revoked
    Revoked,
}

impl StsSession {
    /// Check if session is valid (not expired, not revoked)
    pub fn is_valid(&self) -> bool {
        self.status == SessionStatus::Active && !self.is_expired()
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expiration
    }

    /// Revoke the session
    pub fn revoke(&mut self) {
        self.status = SessionStatus::Revoked;
    }

    /// Update last used timestamp
    pub fn touch(&mut self) {
        self.last_used = Some(chrono::Utc::now());
    }

    /// Update status based on expiration
    pub fn update_status(&mut self) {
        if self.status == SessionStatus::Active && self.is_expired() {
            self.status = SessionStatus::Expired;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};

    fn create_test_session(expiration: chrono::DateTime<chrono::Utc>) -> StsSession {
        StsSession {
            session_token: "token".to_string(),
            access_key_id: "AKIA".to_string(),
            secret_access_key: "secret".to_string(),
            expiration,
            status: SessionStatus::Active,
            assumed_role_arn: None,
            federated_user_name: None,
            principal_arn: None,
            arn: "arn:aws:sts::123456789012:assumed-role/Test/test".to_string(),
            wami_arn: WamiArn::builder()
                .service(crate::arn::Service::Sts)
                .tenant_path(TenantPath::single("root"))
                .wami_instance("123456789012")
                .resource("session", "test")
                .build()
                .unwrap(),
            providers: vec![],
            tenant_id: None,
            created_at: chrono::Utc::now(),
            last_used: None,
        }
    }

    #[test]
    fn test_session_is_valid() {
        let mut session = create_test_session(chrono::Utc::now() + chrono::Duration::hours(1));
        assert!(session.is_valid());

        session.status = SessionStatus::Revoked;
        assert!(!session.is_valid());

        session.status = SessionStatus::Active;
        session.expiration = chrono::Utc::now() - chrono::Duration::hours(1);
        assert!(!session.is_valid());
    }

    #[test]
    fn test_session_is_expired() {
        let expired = create_test_session(chrono::Utc::now() - chrono::Duration::hours(1));
        assert!(expired.is_expired());

        let valid = create_test_session(chrono::Utc::now() + chrono::Duration::hours(1));
        assert!(!valid.is_expired());
    }

    #[test]
    fn test_session_revoke() {
        let mut session = create_test_session(chrono::Utc::now() + chrono::Duration::hours(1));
        assert_eq!(session.status, SessionStatus::Active);
        session.revoke();
        assert_eq!(session.status, SessionStatus::Revoked);
    }

    #[test]
    fn test_session_touch() {
        let mut session = create_test_session(chrono::Utc::now() + chrono::Duration::hours(1));
        assert!(session.last_used.is_none());
        session.touch();
        assert!(session.last_used.is_some());
    }

    #[test]
    fn test_session_update_status() {
        let mut session = create_test_session(chrono::Utc::now() + chrono::Duration::hours(1));
        session.update_status();
        assert_eq!(session.status, SessionStatus::Active);

        session.expiration = chrono::Utc::now() - chrono::Duration::hours(1);
        session.update_status();
        assert_eq!(session.status, SessionStatus::Expired);

        // Revoked sessions shouldn't change
        session.status = SessionStatus::Revoked;
        session.update_status();
        assert_eq!(session.status, SessionStatus::Revoked);
    }
}
