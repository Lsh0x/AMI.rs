//! User Store Trait
//!
//! Focused trait for user-related storage operations

use crate::error::Result;
use crate::types::{PaginationParams, Tag};
use crate::wami::identity::User;
use async_trait::async_trait;

/// Store trait for IAM user operations
#[async_trait]
pub trait UserStore: Send + Sync {
    /// Create a new user
    async fn create_user(&mut self, user: User) -> Result<User>;

    /// Get a user by name
    async fn get_user(&self, user_name: &str) -> Result<Option<User>>;

    /// Update an existing user
    async fn update_user(&mut self, user: User) -> Result<User>;

    /// Delete a user
    async fn delete_user(&mut self, user_name: &str) -> Result<()>;

    /// List users with optional filtering and pagination
    async fn list_users(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<User>, bool, Option<String>)>;

    /// Tag a user
    async fn tag_user(&mut self, user_name: &str, tags: Vec<Tag>) -> Result<()>;

    /// List tags for a user
    async fn list_user_tags(&self, user_name: &str) -> Result<Vec<Tag>>;

    /// Untag a user
    async fn untag_user(&mut self, user_name: &str, tag_keys: Vec<String>) -> Result<()>;
}
