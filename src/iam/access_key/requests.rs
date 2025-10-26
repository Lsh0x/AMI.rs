//! AccessKey Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::AccessKey;
use crate::types::PaginationParams;

// Re-export for convenience
pub use super::model::AccessKeyLastUsed;

/// Request parameters for creating a new access key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccessKeyRequest {
    /// The name of the IAM user to create the access key for
    pub user_name: String,
}

/// Request parameters for updating an access key's status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccessKeyRequest {
    /// The name of the user whose access key should be updated
    pub user_name: String,
    /// The access key ID to update
    pub access_key_id: String,
    /// The new status: "Active" or "Inactive"
    pub status: String,
}

/// Request parameters for listing access keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccessKeysRequest {
    /// The name of the user whose access keys to list
    pub user_name: String,
    /// Pagination parameters
    pub pagination: Option<PaginationParams>,
}

/// Response for listing access keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccessKeysResponse {
    /// List of access keys
    pub access_keys: Vec<AccessKey>,
    /// Whether there are more results
    pub is_truncated: bool,
    /// Marker for the next page
    pub marker: Option<String>,
}
