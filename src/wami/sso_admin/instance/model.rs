//! SSO Instance Model

use serde::{Deserialize, Serialize};

/// Represents an SSO instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoInstance {
    /// The ARN of the SSO instance
    pub instance_arn: String,
    /// The identity store ID
    pub identity_store_id: String,
    /// The name of the SSO instance
    pub name: Option<String>,
    /// The status of the SSO instance
    pub status: String,
    /// The date and time when the instance was created
    pub created_date: chrono::DateTime<chrono::Utc>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
