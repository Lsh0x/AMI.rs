//! User Operations
//!
//! Client methods for managing IAM users

use crate::error::Result;
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::{AmiResponse, Tag};

use super::{builder, model::User, requests::*};

impl<S: Store> IamClient<S> {
    /// Create a new IAM user
    pub async fn create_user(&mut self, request: CreateUserRequest) -> Result<AmiResponse<User>> {
        let account_id = self.account_id().await?;
        let provider = self.cloud_provider();

        // Use builder to construct the user
        // TODO: Extract tenant_id from request or context
        let user = builder::build_user(
            request.user_name,
            request.path,
            request.permissions_boundary,
            request.tags,
            provider.as_ref(),
            &account_id,
            None, // tenant_id - single-tenant mode for now
        );

        let store = self.iam_store().await?;
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
        let account_id = self.account_id().await?;
        let provider = self.cloud_provider();

        let store = self.iam_store().await?;
        // Get existing user
        let user = store.get_user(&request.user_name).await?.ok_or_else(|| {
            crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            }
        })?;

        // Use builder to update user properties
        let updated_user = builder::update_user(
            user,
            request.new_user_name,
            request.new_path,
            provider.as_ref(),
            &account_id,
        );

        let saved_user = store.update_user(updated_user).await?;
        Ok(AmiResponse::success(saved_user))
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
    use crate::store::memory::InMemoryStore;

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
        let user = response.data.unwrap();

        assert_eq!(user.user_name, "test-user");
        assert_eq!(user.path, "/test/");
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut client = create_test_client();
        let request = CreateUserRequest {
            user_name: "test-user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };

        client.create_user(request).await.unwrap();

        let response = client.get_user("test-user".to_string()).await.unwrap();
        let user = response.data.unwrap();

        assert_eq!(user.user_name, "test-user");
    }

    #[tokio::test]
    async fn test_get_nonexistent_user() {
        let mut client = create_test_client();
        let result = client.get_user("nonexistent".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_user() {
        let mut client = create_test_client();
        let request = CreateUserRequest {
            user_name: "test-user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };

        client.create_user(request).await.unwrap();

        let update_request = UpdateUserRequest {
            user_name: "test-user".to_string(),
            new_user_name: Some("updated-user".to_string()),
            new_path: None,
        };

        let response = client.update_user(update_request).await.unwrap();
        let user = response.data.unwrap();

        assert_eq!(user.user_name, "updated-user");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mut client = create_test_client();
        let request = CreateUserRequest {
            user_name: "test-user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };

        client.create_user(request).await.unwrap();
        client.delete_user("test-user".to_string()).await.unwrap();

        let result = client.get_user("test-user".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_users() {
        let mut client = create_test_client();

        for i in 0..3 {
            let request = CreateUserRequest {
                user_name: format!("user-{}", i),
                path: None,
                permissions_boundary: None,
                tags: None,
            };
            client.create_user(request).await.unwrap();
        }

        let response = client.list_users(None).await.unwrap();
        let list_response = response.data.unwrap();

        assert_eq!(list_response.users.len(), 3);
    }

    #[tokio::test]
    async fn test_user_tags() {
        let mut client = create_test_client();
        let request = CreateUserRequest {
            user_name: "test-user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };

        client.create_user(request).await.unwrap();

        let tags = vec![crate::types::Tag {
            key: "Environment".to_string(),
            value: "Test".to_string(),
        }];

        client
            .tag_user("test-user".to_string(), tags.clone())
            .await
            .unwrap();

        let response = client
            .list_user_tags("test-user".to_string())
            .await
            .unwrap();
        let retrieved_tags = response.data.unwrap();

        assert_eq!(retrieved_tags.len(), 1);
        assert_eq!(retrieved_tags[0].key, "Environment");
    }
}
