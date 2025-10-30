//! Permission Set Model

use serde::{Deserialize, Serialize};

/// Represents an SSO permission set
///
/// A permission set defines a collection of permissions that can be assigned to users and groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    /// The ARN of the permission set
    pub permission_set_arn: String,
    /// The name of the permission set
    pub name: String,
    /// A description of the permission set
    pub description: Option<String>,
    /// The length of time that a user can be signed in (ISO-8601 format)
    pub session_duration: Option<String>,
    /// The relay state URL for the application
    pub relay_state: Option<String>,
    /// The SSO instance ARN this permission set belongs to
    pub instance_arn: String,
    /// The date and time when the permission set was created
    pub created_date: chrono::DateTime<chrono::Utc>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
