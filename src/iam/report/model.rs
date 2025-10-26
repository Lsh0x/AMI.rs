//! Report Domain Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// State of credential report generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportState {
    /// Report generation is in progress
    #[serde(rename = "STARTED")]
    Started,
    /// Report generation is complete
    #[serde(rename = "COMPLETE")]
    Complete,
    /// Report generation is in progress
    #[serde(rename = "INPROGRESS")]
    InProgress,
}

/// Stored credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialReport {
    /// The report content in CSV format
    pub content: String,
    /// When the report was generated
    pub generated_time: DateTime<Utc>,
    /// State of the report
    pub state: ReportState,
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
