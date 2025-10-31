//! Signing Certificate Domain Model

use crate::arn::WamiArn;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Signing certificate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CertificateStatus {
    /// Certificate is active
    Active,
    /// Certificate is inactive
    Inactive,
}

/// Signing certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningCertificate {
    /// The name of the user the signing certificate is associated with
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The ID for the signing certificate
    #[serde(rename = "CertificateId")]
    pub certificate_id: String,

    /// The contents of the signing certificate (PEM-encoded)
    #[serde(rename = "CertificateBody")]
    pub certificate_body: String,

    /// The status of the signing certificate
    #[serde(rename = "Status")]
    pub status: CertificateStatus,

    /// The date and time when the signing certificate was uploaded
    #[serde(rename = "UploadDate")]
    pub upload_date: DateTime<Utc>,

    /// The WAMI ARN for cross-provider identification
    pub wami_arn: WamiArn,

    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
