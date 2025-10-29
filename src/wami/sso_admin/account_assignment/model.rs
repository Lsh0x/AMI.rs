//! Account Assignment Model

use serde::{Deserialize, Serialize};

/// Represents an SSO account assignment
///
/// Links a permission set to a principal (user or group) for a specific AWS account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAssignment {
    /// The instance ARN
    pub instance_arn: String,
    /// The account ID
    pub account_id: String,
    /// The permission set ARN
    pub permission_set_arn: String,
    /// The principal type (USER or GROUP)
    pub principal_type: String,
    /// The principal ID
    pub principal_id: String,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
