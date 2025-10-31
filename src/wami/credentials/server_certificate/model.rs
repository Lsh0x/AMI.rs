//! Server Certificate Domain Model

use crate::arn::WamiArn;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Server certificate metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCertificateMetadata {
    /// Path to the server certificate
    #[serde(rename = "Path")]
    pub path: String,

    /// Name of the server certificate
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,

    /// ARN of the server certificate
    #[serde(rename = "Arn")]
    pub arn: String,

    /// Server certificate ID
    #[serde(rename = "ServerCertificateId")]
    pub server_certificate_id: String,

    /// Date and time when the certificate was uploaded
    #[serde(rename = "UploadDate")]
    pub upload_date: DateTime<Utc>,

    /// Date and time when the certificate expires
    #[serde(rename = "Expiration", skip_serializing_if = "Option::is_none")]
    pub expiration: Option<DateTime<Utc>>,
}

/// Server certificate with body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCertificate {
    /// Certificate metadata
    #[serde(rename = "ServerCertificateMetadata")]
    pub server_certificate_metadata: ServerCertificateMetadata,

    /// Contents of the public key certificate in PEM-encoded format
    #[serde(rename = "CertificateBody")]
    pub certificate_body: String,

    /// Contents of the certificate chain in PEM-encoded format
    #[serde(rename = "CertificateChain", skip_serializing_if = "Option::is_none")]
    pub certificate_chain: Option<String>,

    /// Tags associated with the certificate
    #[serde(rename = "Tags", skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<crate::types::Tag>,

    /// The WAMI ARN for cross-provider identification
    pub wami_arn: WamiArn,

    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
