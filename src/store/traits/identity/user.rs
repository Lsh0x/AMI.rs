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

    // Managed policy attachment methods
    /// Attach a managed policy to a user
    async fn attach_user_policy(&mut self, user_name: &str, policy_arn: &str) -> Result<()>;

    /// Detach a managed policy from a user
    async fn detach_user_policy(&mut self, user_name: &str, policy_arn: &str) -> Result<()>;

    /// List all managed policies attached to a user
    async fn list_attached_user_policies(&self, user_name: &str) -> Result<Vec<String>>;

    // Inline policy methods
    /// Put an inline policy on a user
    async fn put_user_policy(
        &mut self,
        user_name: &str,
        policy_name: &str,
        policy_document: String,
    ) -> Result<()>;

    /// Get an inline policy from a user
    async fn get_user_policy(&self, user_name: &str, policy_name: &str) -> Result<Option<String>>;

    /// Delete an inline policy from a user
    async fn delete_user_policy(&mut self, user_name: &str, policy_name: &str) -> Result<()>;

    /// List all inline policy names for a user
    async fn list_user_policies(&self, user_name: &str) -> Result<Vec<String>>;
}
