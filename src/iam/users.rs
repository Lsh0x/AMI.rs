use crate::error::Result;
use crate::types::{AmiResponse, PaginationParams, Tag};
use crate::iam::{IamClient, User};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl IamClient {
    /// Create a new IAM user
    pub async fn create_user(&mut self, request: CreateUserRequest) -> Result<AmiResponse<User>> {
        let user_id = format!("AID{}", uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(17).collect::<String>());
        let arn = format!("arn:aws:iam::123456789012:user/{}", request.user_name);
        
        let user = User {
            user_name: request.user_name.clone(),
            user_id: user_id.clone(),
            arn: arn.clone(),
            path: request.path.unwrap_or_else(|| "/".to_string()),
            create_date: chrono::Utc::now(),
            password_last_used: None,
            permissions_boundary: request.permissions_boundary,
            tags: request.tags.unwrap_or_default(),
        };
        
        self.users.insert(request.user_name, user.clone());
        
        Ok(AmiResponse::success(user))
    }

    /// Delete an IAM user
    pub async fn delete_user(&mut self, user_name: String) -> Result<AmiResponse<()>> {
        if self.users.remove(&user_name).is_some() {
            // Also remove associated access keys
            self.access_keys.retain(|_, key| key.user_name != user_name);
            Ok(AmiResponse::success(()))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("User: {}", user_name) 
            })
        }
    }

    /// Get information about a specific user
    pub async fn get_user(&self, user_name: String) -> Result<AmiResponse<User>> {
        match self.users.get(&user_name) {
            Some(user) => Ok(AmiResponse::success(user.clone())),
            None => Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("User: {}", user_name) 
            })
        }
    }

    /// Update an IAM user
    pub async fn update_user(&mut self, request: UpdateUserRequest) -> Result<AmiResponse<User>> {
        if let Some(mut user) = self.users.remove(&request.user_name) {
            // Update user properties
            if let Some(new_name) = request.new_user_name {
                user.user_name = new_name.clone();
                user.arn = format!("arn:aws:iam::123456789012:user/{}", new_name);
            }
            if let Some(new_path) = request.new_path {
                user.path = new_path;
            }
            
            let updated_user = user.clone();
            self.users.insert(updated_user.user_name.clone(), updated_user.clone());
            
            Ok(AmiResponse::success(updated_user))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("User: {}", request.user_name) 
            })
        }
    }

    /// List all IAM users
    pub async fn list_users(&self, request: Option<ListUsersRequest>) -> Result<AmiResponse<ListUsersResponse>> {
        let mut users: Vec<User> = self.users.values().cloned().collect();
        
        // Apply path prefix filter if specified
        if let Some(req) = &request {
            if let Some(path_prefix) = &req.path_prefix {
                users.retain(|user| user.path.starts_with(path_prefix));
            }
        }
        
        // Sort by user name
        users.sort_by(|a, b| a.user_name.cmp(&b.user_name));
        
        // Apply pagination
        let mut is_truncated = false;
        let mut marker = None;
        
        if let Some(req) = &request {
            if let Some(pagination) = &req.pagination {
                if let Some(max_items) = pagination.max_items {
                    if users.len() > max_items as usize {
                        users.truncate(max_items as usize);
                        is_truncated = true;
                        marker = Some(users.last().unwrap().user_name.clone());
                    }
                }
            }
        }
        
        let response = ListUsersResponse {
            users,
            is_truncated,
            marker,
        };
        
        Ok(AmiResponse::success(response))
    }

    /// List tags for a specific user
    pub async fn list_user_tags(&self, user_name: String) -> Result<AmiResponse<Vec<Tag>>> {
        match self.users.get(&user_name) {
            Some(user) => Ok(AmiResponse::success(user.tags.clone())),
            None => Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("User: {}", user_name) 
            })
        }
    }

    /// Tag a user
    pub async fn tag_user(&mut self, user_name: String, tags: Vec<Tag>) -> Result<AmiResponse<()>> {
        if let Some(user) = self.users.get_mut(&user_name) {
            user.tags.extend(tags);
            Ok(AmiResponse::success(()))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("User: {}", user_name) 
            })
        }
    }

    /// Untag a user
    pub async fn untag_user(&mut self, user_name: String, tag_keys: Vec<String>) -> Result<AmiResponse<()>> {
        if let Some(user) = self.users.get_mut(&user_name) {
            user.tags.retain(|tag| !tag_keys.contains(&tag.key));
            Ok(AmiResponse::success(()))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("User: {}", user_name) 
            })
        }
    }
}
