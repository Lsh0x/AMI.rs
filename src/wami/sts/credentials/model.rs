//! Credentials Domain Model

use crate::arn::WamiArn;
use serde::{Deserialize, Serialize};

/// Temporary AWS credentials
///
/// # Example
///
/// ```rust
/// use wami::sts::Credentials;
/// use chrono::Utc;
///
/// let credentials = Credentials {
///     access_key_id: "ASIAIOSFODNN7EXAMPLE".to_string(),
///     secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
///     session_token: "FwoGZXIvYXdzEBYaDH...".to_string(),
///     expiration: Utc::now() + chrono::Duration::hours(1),
///     arn: "arn:aws:sts::123456789012:assumed-role/MyRole/session".to_string(),
///     wami_arn: "arn:wami:.*:0:wami:123456789012:credentials/session-id".parse().unwrap(),
///     providers: vec![],
///     tenant_id: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// The access key ID
    pub access_key_id: String,
    /// The secret access key
    pub secret_access_key: String,
    /// The session token
    pub session_token: String,
    /// When the credentials expire
    pub expiration: chrono::DateTime<chrono::Utc>,
    /// The native cloud provider ARN (e.g., AWS ARN for assumed role)
    pub arn: String,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: WamiArn,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
    /// Optional tenant ID for multi-tenant isolation
    pub tenant_id: Option<crate::wami::tenant::TenantId>,
}

impl Credentials {
    /// Check if credentials are expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expiration
    }

    /// Time remaining before expiration
    pub fn time_remaining(&self) -> chrono::Duration {
        self.expiration - chrono::Utc::now()
    }

    /// Check if credentials will expire within given duration
    pub fn expires_within(&self, duration: chrono::Duration) -> bool {
        self.time_remaining() < duration
    }
}

/// Type of credential
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialType {
    /// Credentials from assuming a role
    AssumedRole,
    /// Credentials for a federated user
    FederatedUser,
    /// Session token credentials
    SessionToken,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};

    #[test]
    fn test_credentials_is_expired() {
        let wami_arn = WamiArn::builder()
            .service(crate::arn::Service::Sts)
            .tenant_path(TenantPath::single(0))
            .wami_instance("123456789012")
            .resource("credentials", "test")
            .build()
            .unwrap();

        // Expired credentials
        let expired = Credentials {
            access_key_id: "AKIA".to_string(),
            secret_access_key: "secret".to_string(),
            session_token: "token".to_string(),
            expiration: chrono::Utc::now() - chrono::Duration::hours(1),
            arn: "arn:aws:sts::123456789012:assumed-role/Test/test".to_string(),
            wami_arn: wami_arn.clone(),
            providers: vec![],
            tenant_id: None,
        };
        assert!(expired.is_expired());

        // Valid credentials
        let valid = Credentials {
            access_key_id: "AKIA".to_string(),
            secret_access_key: "secret".to_string(),
            session_token: "token".to_string(),
            expiration: chrono::Utc::now() + chrono::Duration::hours(1),
            arn: "arn:aws:sts::123456789012:assumed-role/Test/test".to_string(),
            wami_arn: wami_arn.clone(),
            providers: vec![],
            tenant_id: None,
        };
        assert!(!valid.is_expired());
    }

    #[test]
    fn test_credentials_time_remaining() {
        let wami_arn = WamiArn::builder()
            .service(crate::arn::Service::Sts)
            .tenant_path(TenantPath::single(0))
            .wami_instance("123456789012")
            .resource("credentials", "test")
            .build()
            .unwrap();

        let creds = Credentials {
            access_key_id: "AKIA".to_string(),
            secret_access_key: "secret".to_string(),
            session_token: "token".to_string(),
            expiration: chrono::Utc::now() + chrono::Duration::hours(2),
            arn: "arn:aws:sts::123456789012:assumed-role/Test/test".to_string(),
            wami_arn,
            providers: vec![],
            tenant_id: None,
        };

        let remaining = creds.time_remaining();
        assert!(remaining.num_seconds() > 0);
        assert!(remaining.num_seconds() <= 7200); // Approximately 2 hours
    }

    #[test]
    fn test_credentials_expires_within() {
        let wami_arn = WamiArn::builder()
            .service(crate::arn::Service::Sts)
            .tenant_path(TenantPath::single(0))
            .wami_instance("123456789012")
            .resource("credentials", "test")
            .build()
            .unwrap();

        let creds = Credentials {
            access_key_id: "AKIA".to_string(),
            secret_access_key: "secret".to_string(),
            session_token: "token".to_string(),
            expiration: chrono::Utc::now() + chrono::Duration::minutes(30),
            arn: "arn:aws:sts::123456789012:assumed-role/Test/test".to_string(),
            wami_arn,
            providers: vec![],
            tenant_id: None,
        };

        assert!(creds.expires_within(chrono::Duration::hours(1)));
        assert!(!creds.expires_within(chrono::Duration::minutes(10)));
    }
}
