//! Inline Policy Models

use serde::{Deserialize, Serialize};

/// Represents an inline policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlinePolicy {
    /// The name of the inline policy
    pub policy_name: String,
    /// The policy document in JSON format
    pub policy_document: String,
}
