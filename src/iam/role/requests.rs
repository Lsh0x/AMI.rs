//! Role Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::Role;
use crate::types::{PaginationParams, Tag};

/// Request parameters for creating a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    /// The name of the role
    pub role_name: String,
    /// The trust relationship policy document
    pub assume_role_policy_document: String,
    /// The path to the role
    pub path: Option<String>,
    /// A description of the role
    pub description: Option<String>,
    /// The maximum session duration in seconds (1h to 12h)
    pub max_session_duration: Option<i32>,
    /// The ARN of the policy used to set the permissions boundary
    pub permissions_boundary: Option<String>,
    /// Tags to attach to the role
    pub tags: Option<Vec<Tag>>,
}

/// Request parameters for updating a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    /// The name of the role to update
    pub role_name: String,
    /// New description
    pub description: Option<String>,
    /// New maximum session duration
    pub max_session_duration: Option<i32>,
}

/// Request parameters for listing roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolesRequest {
    /// Path prefix for filtering roles
    pub path_prefix: Option<String>,
    /// Pagination parameters
    pub pagination: Option<PaginationParams>,
}

/// Response for listing roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolesResponse {
    /// List of roles
    pub roles: Vec<Role>,
    /// Whether the results are truncated
    pub is_truncated: bool,
    /// Marker for pagination
    pub marker: Option<String>,
}
