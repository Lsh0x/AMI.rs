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
}
