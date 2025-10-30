//! Group Store Trait
//!
//! Focused trait for group-related storage operations

use crate::error::Result;
use crate::types::PaginationParams;
use crate::wami::identity::Group;
use async_trait::async_trait;

/// Store trait for IAM group operations
#[async_trait]
pub trait GroupStore: Send + Sync {
    /// Create a new group
    async fn create_group(&mut self, group: Group) -> Result<Group>;

    /// Get a group by name
    async fn get_group(&self, group_name: &str) -> Result<Option<Group>>;

    /// Update an existing group
    async fn update_group(&mut self, group: Group) -> Result<Group>;

    /// Delete a group
    async fn delete_group(&mut self, group_name: &str) -> Result<()>;

    /// List groups with optional filtering and pagination
    async fn list_groups(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Group>, bool, Option<String>)>;

    /// List groups for a specific user
    async fn list_groups_for_user(&self, user_name: &str) -> Result<Vec<Group>>;

    /// Add a user to a group
    async fn add_user_to_group(&mut self, group_name: &str, user_name: &str) -> Result<()>;

    /// Remove a user from a group
    async fn remove_user_from_group(&mut self, group_name: &str, user_name: &str) -> Result<()>;

    // Managed policy attachment methods
    /// Attach a managed policy to a group
    async fn attach_group_policy(&mut self, group_name: &str, policy_arn: &str) -> Result<()>;

    /// Detach a managed policy from a group
    async fn detach_group_policy(&mut self, group_name: &str, policy_arn: &str) -> Result<()>;

    /// List all managed policies attached to a group
    async fn list_attached_group_policies(&self, group_name: &str) -> Result<Vec<String>>;

    // Inline policy methods
    /// Put an inline policy on a group
    async fn put_group_policy(
        &mut self,
        group_name: &str,
        policy_name: &str,
        policy_document: String,
    ) -> Result<()>;

    /// Get an inline policy from a group
    async fn get_group_policy(&self, group_name: &str, policy_name: &str)
        -> Result<Option<String>>;

    /// Delete an inline policy from a group
    async fn delete_group_policy(&mut self, group_name: &str, policy_name: &str) -> Result<()>;

    /// List all inline policy names for a group
    async fn list_group_policies(&self, group_name: &str) -> Result<Vec<String>>;
}
