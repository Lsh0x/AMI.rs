//! Service Credential Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpecificCredential {
    /// The name of the IAM user associated with the credential
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier for the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,

    /// The generated username for the service
    #[serde(rename = "ServiceUserName")]
    pub service_user_name: String,

    /// The generated password for the service (only returned on creation)
    #[serde(rename = "ServicePassword", skip_serializing_if = "Option::is_none")]
    pub service_password: Option<String>,

    /// The name of the service
    #[serde(rename = "ServiceName")]
    pub service_name: String,

    /// The date and time when the credential was created
    #[serde(rename = "CreateDate")]
    pub create_date: DateTime<Utc>,

    /// The status of the credential (Active or Inactive)
    #[serde(rename = "Status")]
    pub status: String,

    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,

    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Metadata about a service-specific credential (without password)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpecificCredentialMetadata {
    /// The name of the IAM user associated with the credential
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier for the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,

    /// The generated username for the service
    #[serde(rename = "ServiceUserName")]
    pub service_user_name: String,

    /// The name of the service
    #[serde(rename = "ServiceName")]
    pub service_name: String,

    /// The date and time when the credential was created
    #[serde(rename = "CreateDate")]
    pub create_date: DateTime<Utc>,

    /// The status of the credential (Active or Inactive)
    #[serde(rename = "Status")]
    pub status: String,
}
