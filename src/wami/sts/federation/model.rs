//! Federation Domain Model

use serde::{Deserialize, Serialize};

/// Information about a federated user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedUser {
    /// The ARN of the federated user
    pub arn: String,
    /// The unique identifier of the federated user
    pub federated_user_id: String,
}
