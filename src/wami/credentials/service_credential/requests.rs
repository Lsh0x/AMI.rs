//! Service Credential Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::*;

/// Request to create a service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceSpecificCredentialRequest {
    /// The name of the IAM user to associate with the credential
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The name of the AWS service (e.g., "codecommit.amazonaws.com")
    #[serde(rename = "ServiceName")]
    pub service_name: String,
}

/// Response from creating a service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceSpecificCredentialResponse {
    /// The created credential with password
    #[serde(rename = "ServiceSpecificCredential")]
    pub service_specific_credential: ServiceSpecificCredential,
}

/// Request to delete a service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteServiceSpecificCredentialRequest {
    /// The name of the IAM user
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier of the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,
}

/// Request to list service-specific credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServiceSpecificCredentialsRequest {
    /// The name of the IAM user (optional, lists all if not provided)
    #[serde(rename = "UserName", skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    /// Filter by service name (optional)
    #[serde(rename = "ServiceName", skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
}

/// Response from listing service-specific credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServiceSpecificCredentialsResponse {
    /// List of credential metadata
    #[serde(rename = "ServiceSpecificCredentials")]
    pub service_specific_credentials: Vec<ServiceSpecificCredentialMetadata>,
}

/// Request to reset a service-specific credential password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetServiceSpecificCredentialRequest {
    /// The name of the IAM user
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier of the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,
}

/// Response from resetting a service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetServiceSpecificCredentialResponse {
    /// The credential with new password
    #[serde(rename = "ServiceSpecificCredential")]
    pub service_specific_credential: ServiceSpecificCredential,
}

/// Request to update a service-specific credential status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceSpecificCredentialRequest {
    /// The name of the IAM user
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier of the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,

    /// The new status (Active or Inactive)
    #[serde(rename = "Status")]
    pub status: String,
}
