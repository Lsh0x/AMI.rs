//! Server Certificate Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::*;

/// Request to upload a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadServerCertificateRequest {
    /// Name for the server certificate
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,

    /// Contents of the public key certificate in PEM-encoded format
    #[serde(rename = "CertificateBody")]
    pub certificate_body: String,

    /// Contents of the private key in PEM-encoded format
    #[serde(rename = "PrivateKey")]
    pub private_key: String,

    /// Contents of the certificate chain in PEM-encoded format
    #[serde(rename = "CertificateChain", skip_serializing_if = "Option::is_none")]
    pub certificate_chain: Option<String>,

    /// Path for the server certificate
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Tags to attach to the certificate
    #[serde(rename = "Tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<crate::types::Tag>>,
}

/// Response from uploading a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadServerCertificateResponse {
    /// Information about the uploaded certificate
    #[serde(rename = "ServerCertificateMetadata")]
    pub server_certificate_metadata: ServerCertificateMetadata,

    /// Tags attached to the certificate
    #[serde(rename = "Tags", skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<crate::types::Tag>,
}

/// Request to get a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServerCertificateRequest {
    /// Name of the server certificate to retrieve
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,
}

/// Response from getting a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServerCertificateResponse {
    /// The server certificate
    #[serde(rename = "ServerCertificate")]
    pub server_certificate: ServerCertificate,
}

/// Request to list server certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServerCertificatesRequest {
    /// Path prefix to filter certificates
    #[serde(rename = "PathPrefix", skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,

    /// Marker for pagination
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,

    /// Maximum number of items to return
    #[serde(rename = "MaxItems", skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
}

/// Response from listing server certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServerCertificatesResponse {
    /// List of server certificate metadata
    #[serde(rename = "ServerCertificateMetadataList")]
    pub server_certificate_metadata_list: Vec<ServerCertificateMetadata>,

    /// Indicates whether the list is truncated
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,

    /// Marker for next page
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// Request to delete a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteServerCertificateRequest {
    /// Name of the server certificate to delete
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,
}

/// Request to update a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerCertificateRequest {
    /// Current name of the server certificate
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,

    /// New name for the server certificate
    #[serde(
        rename = "NewServerCertificateName",
        skip_serializing_if = "Option::is_none"
    )]
    pub new_server_certificate_name: Option<String>,

    /// New path for the server certificate
    #[serde(rename = "NewPath", skip_serializing_if = "Option::is_none")]
    pub new_path: Option<String>,
}
