//! SSO Application Model

use serde::{Deserialize, Serialize};

/// Represents an SSO-enabled application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    /// The ARN of the application
    pub application_arn: String,
    /// The instance ARN this application belongs to
    pub instance_arn: String,
    /// The name of the application
    pub name: String,
    /// A description of the application
    pub description: Option<String>,
    /// The application provider ARN
    pub application_provider_arn: String,
    /// The portal URL for the application
    pub portal_url: Option<String>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
