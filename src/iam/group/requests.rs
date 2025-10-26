//! Group Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::Group;
use crate::types::{PaginationParams, Tag};

/// Parameters for creating a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub group_name: String,
    pub path: Option<String>,
    pub tags: Option<Vec<Tag>>,
}

/// Parameters for updating a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub group_name: String,
    pub new_group_name: Option<String>,
    pub new_path: Option<String>,
}

/// Parameters for listing groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGroupsRequest {
    pub path_prefix: Option<String>,
    pub pagination: Option<PaginationParams>,
}

/// Response for listing groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGroupsResponse {
    pub groups: Vec<Group>,
    pub is_truncated: bool,
    pub marker: Option<String>,
}
