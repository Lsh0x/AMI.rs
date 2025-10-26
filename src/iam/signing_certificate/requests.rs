//! Signing Certificate Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::*;

/// Request to upload a signing certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSigningCertificateRequest {
    /// The name of the user the signing certificate is for
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The contents of the signing certificate (PEM-encoded X.509 certificate)
    #[serde(rename = "CertificateBody")]
    pub certificate_body: String,
}

/// Response from uploading a signing certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSigningCertificateResponse {
    /// Information about the uploaded signing certificate
    #[serde(rename = "Certificate")]
    pub certificate: SigningCertificate,
}

/// Request to delete a signing certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSigningCertificateRequest {
    /// The name of the user the signing certificate belongs to
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The ID of the signing certificate to delete
    #[serde(rename = "CertificateId")]
    pub certificate_id: String,
}

/// Request to list signing certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSigningCertificatesRequest {
    /// The name of the user to list signing certificates for
    #[serde(rename = "UserName", skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// Response from listing signing certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSigningCertificatesResponse {
    /// List of signing certificates
    #[serde(rename = "Certificates")]
    pub certificates: Vec<SigningCertificate>,

    /// Whether the results are truncated
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,

    /// Marker for pagination
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// Request to update a signing certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSigningCertificateRequest {
    /// The name of the user the signing certificate belongs to
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The ID of the signing certificate to update
    #[serde(rename = "CertificateId")]
    pub certificate_id: String,

    /// The new status for the signing certificate
    #[serde(rename = "Status")]
    pub status: CertificateStatus,
}
