//! Credentials Domain Model

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
///     wami_arn: "arn:wami:sts:tenant-hash:credentials/session-id".to_string(),
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
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
    /// Optional tenant ID for multi-tenant isolation
    pub tenant_id: Option<crate::tenant::TenantId>,
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
