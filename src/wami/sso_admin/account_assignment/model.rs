//! Account Assignment Model

use serde::{Deserialize, Serialize};

/// Represents an SSO account assignment
///
/// Links a permission set to a principal (user or group) for a specific AWS account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAssignment {
    /// Unique identifier for this assignment
    pub assignment_id: String,
    /// The instance ARN
    pub instance_arn: String,
    /// The account ID (target account)
    pub account_id: String,
    /// The permission set ARN
    pub permission_set_arn: String,
    /// The principal type (USER or GROUP)
    pub principal_type: String,
    /// The principal ID
    pub principal_id: String,
    /// The target ID (usually same as account_id)
    pub target_id: String,
    /// The target type (usually AWS_ACCOUNT)
    pub target_type: String,
    /// When this assignment was created
    pub created_date: chrono::DateTime<chrono::Utc>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
