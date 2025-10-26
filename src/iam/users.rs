use crate::error::Result;
use crate::iam::{IamClient, User};
use crate::provider::ResourceType;
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
        let provider = store.cloud_provider();

        // Use provider for ID and ARN generation
        let user_id = provider.generate_resource_id(ResourceType::User);
        let path = request.path.unwrap_or_else(|| "/".to_string());
        let arn = provider.generate_resource_identifier(
            ResourceType::User,
            account_id,
            &path,
            &request.user_name,
        );

        let user = User {
            user_name: request.user_name.clone(),
            user_id: user_id.clone(),
            arn: arn.clone(),
            path,
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
        let provider = store.cloud_provider();
        let account_id = store.account_id();

        // Get existing user
        let mut user = store.get_user(&request.user_name).await?.ok_or_else(|| {
            crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            }
        })?;

        // Update user properties
        if let Some(new_name) = request.new_user_name {
            user.user_name = new_name.clone();
            // Use provider for ARN generation
            user.arn = provider.generate_resource_identifier(
                ResourceType::User,
                account_id,
                &user.path,
                &new_name,
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::in_memory::InMemoryStore;

    fn create_test_client() -> IamClient<InMemoryStore> {
        let store = InMemoryStore::new();
        IamClient::new(store)
    }

    #[tokio::test]
    async fn test_create_user() {
        let mut client = create_test_client();

        let request = CreateUserRequest {
            user_name: "test-user".to_string(),
            path: Some("/test/".to_string()),
            permissions_boundary: None,
            tags: None,
        };

        let response = client.create_user(request).await.unwrap();
        assert!(response.success);

        let user = response.data.unwrap();
        assert_eq!(user.user_name, "test-user");
        assert_eq!(user.path, "/test/");
        assert!(user.arn.contains("test-user"));
        assert!(user.user_id.starts_with("AID"));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut client = create_test_client();

        // Create a user first
        let create_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: Some("/".to_string()),
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(create_request).await.unwrap();

        // Get the user
        let response = client.get_user("alice".to_string()).await.unwrap();
        assert!(response.success);

        let user = response.data.unwrap();
        assert_eq!(user.user_name, "alice");
    }

    #[tokio::test]
    async fn test_get_nonexistent_user() {
        let mut client = create_test_client();

        let result = client.get_user("nonexistent".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mut client = create_test_client();

        // Create a user
        let create_request = CreateUserRequest {
            user_name: "bob".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(create_request).await.unwrap();

        // Delete the user
        let response = client.delete_user("bob".to_string()).await.unwrap();
        assert!(response.success);

        // Verify user is deleted
        let result = client.get_user("bob".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_user() {
        let mut client = create_test_client();

        // Create a user
        let create_request = CreateUserRequest {
            user_name: "charlie".to_string(),
            path: Some("/old/".to_string()),
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(create_request).await.unwrap();

        // Update the user
        let update_request = UpdateUserRequest {
            user_name: "charlie".to_string(),
            new_user_name: None,
            new_path: Some("/new/".to_string()),
        };
        let response = client.update_user(update_request).await.unwrap();
        assert!(response.success);

        let user = response.data.unwrap();
        assert_eq!(user.path, "/new/");
    }

    #[tokio::test]
    async fn test_list_users() {
        let mut client = create_test_client();

        // Create multiple users
        for name in &["user1", "user2", "user3"] {
            let request = CreateUserRequest {
                user_name: name.to_string(),
                path: Some("/".to_string()),
                permissions_boundary: None,
                tags: None,
            };
            client.create_user(request).await.unwrap();
        }

        // List users
        let response = client.list_users(None).await.unwrap();
        assert!(response.success);

        let list_response = response.data.unwrap();
        assert_eq!(list_response.users.len(), 3);
        assert!(!list_response.is_truncated);
    }

    #[tokio::test]
    async fn test_user_tags() {
        let mut client = create_test_client();

        // Create a user
        let create_request = CreateUserRequest {
            user_name: "tagged-user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(create_request).await.unwrap();

        // Add tags
        let tags = vec![
            Tag {
                key: "Environment".to_string(),
                value: "Production".to_string(),
            },
            Tag {
                key: "Team".to_string(),
                value: "Engineering".to_string(),
            },
        ];
        let response = client
            .tag_user("tagged-user".to_string(), tags)
            .await
            .unwrap();
        assert!(response.success);

        // List tags
        let response = client
            .list_user_tags("tagged-user".to_string())
            .await
            .unwrap();
        assert!(response.success);

        let tags = response.data.unwrap();
        assert_eq!(tags.len(), 2);

        // Remove a tag
        let response = client
            .untag_user("tagged-user".to_string(), vec!["Environment".to_string()])
            .await
            .unwrap();
        assert!(response.success);

        // Verify tag was removed
        let response = client
            .list_user_tags("tagged-user".to_string())
            .await
            .unwrap();
        let tags = response.data.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].key, "Team");
    }
}
