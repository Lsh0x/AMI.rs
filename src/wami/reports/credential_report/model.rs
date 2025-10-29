//! Credential Report Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Credential report status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CredentialReportStatus {
    InProgress,
    Complete,
    Failed,
}

/// State of credential report generation (alias for compatibility)
pub type ReportState = CredentialReportStatus;

/// Credential report entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialReport {
    /// When the report was generated
    pub generated_time: DateTime<Utc>,

    /// CSV content of the report
    pub report_content: Vec<u8>,

    /// Report format
    pub report_format: String,
}

impl CredentialReport {
    /// Create a new credential report
    pub fn new(report_content: Vec<u8>) -> Self {
        Self {
            generated_time: Utc::now(),
            report_content,
            report_format: "text/csv".to_string(),
        }
    }
}

/// Account summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountSummaryMap {
    /// Number of users
    #[serde(rename = "Users")]
    pub users: u32,

    /// Quota for users
    #[serde(rename = "UsersQuota")]
    pub users_quota: u32,

    /// Number of groups
    #[serde(rename = "Groups")]
    pub groups: u32,

    /// Quota for groups
    #[serde(rename = "GroupsQuota")]
    pub groups_quota: u32,

    /// Number of roles
    #[serde(rename = "Roles")]
    pub roles: u32,

    /// Quota for roles
    #[serde(rename = "RolesQuota")]
    pub roles_quota: u32,

    /// Number of policies
    #[serde(rename = "Policies")]
    pub policies: u32,

    /// Quota for policies
    #[serde(rename = "PoliciesQuota")]
    pub policies_quota: u32,

    /// Number of MFA devices
    #[serde(rename = "MFADevices")]
    pub mfa_devices: u32,

    /// Number of MFA devices in use
    #[serde(rename = "MFADevicesInUse")]
    pub mfa_devices_in_use: u32,

    /// Number of server certificates
    #[serde(rename = "ServerCertificates")]
    pub server_certificates: u32,

    /// Quota for server certificates
    #[serde(rename = "ServerCertificatesQuota")]
    pub server_certificates_quota: u32,

    /// Number of access keys per user quota
    #[serde(rename = "AccessKeysPerUserQuota")]
    pub access_keys_per_user_quota: u32,

    /// Number of signing certificates per user quota
    #[serde(rename = "SigningCertificatesPerUserQuota")]
    pub signing_certificates_per_user_quota: u32,
}
