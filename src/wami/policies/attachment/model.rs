//! Policy Attachment Models

use serde::{Deserialize, Serialize};

/// Represents an attached managed policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachedPolicy {
    /// The friendly name of the attached policy
    pub policy_name: String,
    /// The Amazon Resource Name (ARN) that identifies the policy
    pub policy_arn: String,
}
