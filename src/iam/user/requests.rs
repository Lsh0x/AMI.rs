//! User Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::User;
use crate::types::{PaginationParams, Tag};

/// Parameters for creating a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub user_name: String,
    pub path: Option<String>,
    pub permissions_boundary: Option<String>,
    pub tags: Option<Vec<Tag>>,
}

/// Parameters for updating a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub user_name: String,
    pub new_user_name: Option<String>,
    pub new_path: Option<String>,
}

/// Parameters for listing users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersRequest {
    pub path_prefix: Option<String>,
    pub pagination: Option<PaginationParams>,
}

/// Response for listing users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersResponse {
    pub users: Vec<User>,
    pub is_truncated: bool,
    pub marker: Option<String>,
}
