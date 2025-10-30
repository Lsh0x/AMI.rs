//! Role Store Trait
//!
//! Focused trait for role-related storage operations

use crate::error::Result;
use crate::types::PaginationParams;
use crate::wami::identity::Role;
use async_trait::async_trait;

/// Store trait for IAM role operations
#[async_trait]
pub trait RoleStore: Send + Sync {
    /// Create a new role
    async fn create_role(&mut self, role: Role) -> Result<Role>;

    /// Get a role by name
    async fn get_role(&self, role_name: &str) -> Result<Option<Role>>;

    /// Update an existing role
    async fn update_role(&mut self, role: Role) -> Result<Role>;

    /// Delete a role
    async fn delete_role(&mut self, role_name: &str) -> Result<()>;

    /// List roles with optional filtering and pagination
    async fn list_roles(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Role>, bool, Option<String>)>;

    // Managed policy attachment methods
    /// Attach a managed policy to a role
    async fn attach_role_policy(&mut self, role_name: &str, policy_arn: &str) -> Result<()>;

    /// Detach a managed policy from a role
    async fn detach_role_policy(&mut self, role_name: &str, policy_arn: &str) -> Result<()>;

    /// List all managed policies attached to a role
    async fn list_attached_role_policies(&self, role_name: &str) -> Result<Vec<String>>;

    // Inline policy methods
    /// Put an inline policy on a role
    async fn put_role_policy(
        &mut self,
        role_name: &str,
        policy_name: &str,
        policy_document: String,
    ) -> Result<()>;

    /// Get an inline policy from a role
    async fn get_role_policy(&self, role_name: &str, policy_name: &str) -> Result<Option<String>>;

    /// Delete an inline policy from a role
    async fn delete_role_policy(&mut self, role_name: &str, policy_name: &str) -> Result<()>;

    /// List all inline policy names for a role
    async fn list_role_policies(&self, role_name: &str) -> Result<Vec<String>>;
}
