//! Assume Role Domain Model

use serde::{Deserialize, Serialize};

/// Information about an assumed role user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumedRoleUser {
    /// The unique identifier of the assumed role user
    pub assumed_role_id: String,
    /// The ARN of the assumed role
    pub arn: String,
}
