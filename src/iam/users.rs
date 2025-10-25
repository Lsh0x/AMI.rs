use crate::error::Result;
use crate::iam::{IamClient, User};
use crate::store::{IamStore, Store};
use crate::types::{AmiResponse, PaginationParams, Tag};
use serde::{Deserialize, Serialize};

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

impl<S: Store> IamClient<S> {
    /// Create a new IAM user
    pub async fn create_user(&mut self, request: CreateUserRequest) -> Result<AmiResponse<User>> {
        let store = self.iam_store().await?;
        let account_id = store.account_id();

        let user_id = format!(
            "AID{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );
        let arn = format!("arn:aws:iam::{}:user/{}", account_id, request.user_name);

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

        let created_user = store.create_user(user).await?;

        Ok(AmiResponse::success(created_user))
    }

    /// Delete an IAM user
    pub async fn delete_user(&mut self, user_name: String) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;
        store.delete_user(&user_name).await?;
        Ok(AmiResponse::success(()))
    }

    /// Get information about a specific user
    pub async fn get_user(&mut self, user_name: String) -> Result<AmiResponse<User>> {
        let store = self.iam_store().await?;
        match store.get_user(&user_name).await? {
            Some(user) => Ok(AmiResponse::success(user)),
            None => Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", user_name),
            }),
        }
    }

    /// Update an IAM user
    pub async fn update_user(&mut self, request: UpdateUserRequest) -> Result<AmiResponse<User>> {
        let store = self.iam_store().await?;

        // Get existing user
        let mut user = store.get_user(&request.user_name).await?.ok_or_else(|| {
            crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            }
        })?;

        // Update user properties
        if let Some(new_name) = request.new_user_name {
            user.user_name = new_name.clone();
            user.arn = format!("arn:aws:iam::{}:user/{}", store.account_id(), new_name);
        }
        if let Some(new_path) = request.new_path {
            user.path = new_path;
        }

        let updated_user = store.update_user(user).await?;
        Ok(AmiResponse::success(updated_user))
    }

    /// List all IAM users
    pub async fn list_users(
        &mut self,
        request: Option<ListUsersRequest>,
    ) -> Result<AmiResponse<ListUsersResponse>> {
        let store = self.iam_store().await?;

        let path_prefix = request.as_ref().and_then(|r| r.path_prefix.as_deref());
        let pagination = request.as_ref().and_then(|r| r.pagination.as_ref());

        let (users, is_truncated, marker) = store.list_users(path_prefix, pagination).await?;

        let response = ListUsersResponse {
            users,
            is_truncated,
            marker,
        };

        Ok(AmiResponse::success(response))
    }

    /// List tags for a specific user
    pub async fn list_user_tags(&mut self, user_name: String) -> Result<AmiResponse<Vec<Tag>>> {
        let store = self.iam_store().await?;
        let tags = store.list_user_tags(&user_name).await?;
        Ok(AmiResponse::success(tags))
    }

    /// Tag a user
    pub async fn tag_user(&mut self, user_name: String, tags: Vec<Tag>) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;
        store.tag_user(&user_name, tags).await?;
        Ok(AmiResponse::success(()))
    }

    /// Untag a user
    pub async fn untag_user(
        &mut self,
        user_name: String,
        tag_keys: Vec<String>,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;
        store.untag_user(&user_name, tag_keys).await?;
        Ok(AmiResponse::success(()))
    }
}
