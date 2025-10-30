//! User Service
//!
//! Orchestrates user management operations by combining wami builders with store persistence.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::UserStore;
use crate::types::Tag;
use crate::wami::identity::user::{
    builder as user_builder, CreateUserRequest, ListUsersRequest, UpdateUserRequest, User,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM users
///
/// Provides high-level operations that combine wami pure functions with store persistence.
/// Supports fluent provider chaining for multi-cloud scenarios.
///
/// # Example
///
/// ```rust
/// use wami::service::UserService;
/// use wami::store::memory::InMemoryWamiStore;
/// use wami::wami::identity::user::CreateUserRequest;
/// use std::sync::{Arc, RwLock};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
/// let service = UserService::new(store, "123456789012".to_string());
///
/// // Create user with default AWS provider
/// let request = CreateUserRequest {
///     user_name: "alice".to_string(),
///     path: Some("/engineering/".to_string()),
///     permissions_boundary: None,
///     tags: None,
/// };
/// let user = service.create_user(request).await?;
/// # Ok(())
/// # }
/// ```
pub struct UserService<S> {
    store: Arc<RwLock<S>>,
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: UserStore> UserService<S> {
    /// Create a new UserService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>, account_id: String) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
            account_id,
        }
    }

    /// Returns a new service instance with different provider (cheap clone - all Arc)
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::service::UserService;
    /// use wami::provider::GcpProvider;
    /// use std::sync::{Arc, RwLock};
    /// # use wami::store::memory::InMemoryWamiStore;
    ///
    /// # let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    /// let service = UserService::new(store, "123456789012".to_string());
    ///
    /// // Override to GCP for one operation
    /// let gcp_provider = Arc::new(GcpProvider::new("my-project-id"));
    /// let gcp_service = service.with_provider(gcp_provider);
    /// ```
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
            account_id: self.account_id.clone(),
        }
    }

    /// Create a new user
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        // Use wami builder to create user with current provider
        let user = user_builder::build_user(
            request.user_name,
            request.path,
            &*self.provider,
            &self.account_id,
        );

        // Apply permissions boundary if specified
        let user = if let Some(boundary_arn) = request.permissions_boundary {
            user_builder::set_permissions_boundary(user, boundary_arn)
        } else {
            user
        };

        // Apply tags if specified
        let user = if let Some(tags) = request.tags {
            user_builder::add_tags(user, tags)
        } else {
            user
        };

        // Store it
        self.store.write().unwrap().create_user(user).await
    }

    /// Get a user by name
    pub async fn get_user(&self, user_name: &str) -> Result<Option<User>> {
        self.store.read().unwrap().get_user(user_name).await
    }

    /// Update a user
    pub async fn update_user(&self, request: UpdateUserRequest) -> Result<User> {
        // Get existing user
        let mut user = self
            .store
            .read()
            .unwrap()
            .get_user(&request.user_name)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            })?;

        // Apply updates using builder functions
        if let Some(new_user_name) = request.new_user_name {
            user = user_builder::update_user_name(user, new_user_name);
        }

        if let Some(new_path) = request.new_path {
            user = user_builder::update_user_path(user, new_path);
        }

        // Store updated user
        self.store.write().unwrap().update_user(user).await
    }

    /// Delete a user
    pub async fn delete_user(&self, user_name: &str) -> Result<()> {
        self.store.write().unwrap().delete_user(user_name).await
    }

    /// List users with optional filtering
    pub async fn list_users(
        &self,
        request: ListUsersRequest,
    ) -> Result<(Vec<User>, bool, Option<String>)> {
        self.store
            .read()
            .unwrap()
            .list_users(request.path_prefix.as_deref(), request.pagination.as_ref())
            .await
    }

    /// Tag a user
    pub async fn tag_user(&self, user_name: &str, tags: Vec<Tag>) -> Result<()> {
        self.store.write().unwrap().tag_user(user_name, tags).await
    }

    /// List tags for a user
    pub async fn list_user_tags(&self, user_name: &str) -> Result<Vec<Tag>> {
        self.store.read().unwrap().list_user_tags(user_name).await
    }

    /// Untag a user
    pub async fn untag_user(&self, user_name: &str, tag_keys: Vec<String>) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .untag_user(user_name, tag_keys)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> UserService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        UserService::new(store, "123456789012".to_string())
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let service = setup_service();

        let request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: Some("/engineering/".to_string()),
            permissions_boundary: None,
            tags: None,
        };

        let user = service.create_user(request).await.unwrap();
        assert_eq!(user.user_name, "alice");
        assert_eq!(user.path, "/engineering/");

        let retrieved = service.get_user("alice").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_name, "alice");
    }

    #[tokio::test]
    async fn test_update_user() {
        let service = setup_service();

        // Create user
        let create_request = CreateUserRequest {
            user_name: "bob".to_string(),
            path: Some("/".to_string()),
            permissions_boundary: None,
            tags: None,
        };
        service.create_user(create_request).await.unwrap();

        // Update user
        let update_request = UpdateUserRequest {
            user_name: "bob".to_string(),
            new_user_name: Some("robert".to_string()),
            new_path: Some("/admin/".to_string()),
        };
        let updated = service.update_user(update_request).await.unwrap();
        assert_eq!(updated.user_name, "robert");
        assert_eq!(updated.path, "/admin/");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let service = setup_service();

        let request = CreateUserRequest {
            user_name: "charlie".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        service.create_user(request).await.unwrap();

        service.delete_user("charlie").await.unwrap();

        let retrieved = service.get_user("charlie").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_users() {
        let service = setup_service();

        // Create multiple users
        for name in ["user1", "user2", "user3"] {
            let request = CreateUserRequest {
                user_name: name.to_string(),
                path: Some("/test/".to_string()),
                permissions_boundary: None,
                tags: None,
            };
            service.create_user(request).await.unwrap();
        }

        let list_request = ListUsersRequest {
            path_prefix: Some("/test/".to_string()),
            pagination: None,
        };
        let (users, _, _) = service.list_users(list_request).await.unwrap();
        assert_eq!(users.len(), 3);
    }

    #[tokio::test]
    async fn test_tag_operations() {
        let service = setup_service();

        let request = CreateUserRequest {
            user_name: "tagged_user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        service.create_user(request).await.unwrap();

        // Tag user
        let tags = vec![Tag {
            key: "Environment".to_string(),
            value: "Production".to_string(),
        }];
        service.tag_user("tagged_user", tags).await.unwrap();

        // List tags
        let retrieved_tags = service.list_user_tags("tagged_user").await.unwrap();
        assert_eq!(retrieved_tags.len(), 1);
        assert_eq!(retrieved_tags[0].key, "Environment");

        // Untag
        service
            .untag_user("tagged_user", vec!["Environment".to_string()])
            .await
            .unwrap();

        let tags_after = service.list_user_tags("tagged_user").await.unwrap();
        assert_eq!(tags_after.len(), 0);
    }

    #[tokio::test]
    async fn test_with_provider() {
        let service = setup_service();
        let gcp_provider = Arc::new(crate::provider::GcpProvider::new("test-project"));

        let gcp_service = service.with_provider(gcp_provider);

        // Both services should work independently
        let request1 = CreateUserRequest {
            user_name: "aws_user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        let aws_user = service.create_user(request1).await.unwrap();

        let request2 = CreateUserRequest {
            user_name: "gcp_user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        let gcp_user = gcp_service.create_user(request2).await.unwrap();

        // AWS user should have AWS-style ARN
        assert!(aws_user.arn.contains("arn:aws:iam"));

        // GCP user should have GCP-style service account identifier
        assert!(gcp_user.arn.contains("projects/") && gcp_user.arn.contains("serviceAccounts/"));
    }
}
