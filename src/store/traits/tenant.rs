//! Tenant Store Trait

use crate::error::Result;
use crate::tenant::{Tenant, TenantId, TenantQuotas, TenantUsage};
use async_trait::async_trait;

/// Tenant actions for permission checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantAction {
    /// Read tenant information
    Read,
    /// Update tenant
    Update,
    /// Delete tenant
    Delete,
    /// Create sub-tenant
    CreateSubTenant,
    /// Manage users in tenant
    ManageUsers,
    /// Manage roles in tenant
    ManageRoles,
    /// Manage policies in tenant
    ManagePolicies,
}

/// Trait for tenant storage operations
#[async_trait]
pub trait TenantStore: Send + Sync {
    // Basic CRUD operations
    /// Create a new tenant
    async fn create_tenant(&mut self, tenant: Tenant) -> Result<Tenant>;

    /// Get a tenant by ID
    async fn get_tenant(&self, tenant_id: &TenantId) -> Result<Option<Tenant>>;

    /// Update a tenant
    async fn update_tenant(&mut self, tenant: Tenant) -> Result<Tenant>;

    /// Delete a tenant
    async fn delete_tenant(&mut self, tenant_id: &TenantId) -> Result<()>;

    /// List all tenants
    async fn list_tenants(&self) -> Result<Vec<Tenant>>;

    // Hierarchy operations
    /// List direct child tenants
    async fn list_child_tenants(&self, parent_id: &TenantId) -> Result<Vec<Tenant>>;

    /// Get all ancestor tenants
    async fn get_ancestors(&self, tenant_id: &TenantId) -> Result<Vec<Tenant>>;

    /// Get all descendant tenant IDs
    async fn get_descendants(&self, tenant_id: &TenantId) -> Result<Vec<TenantId>>;

    // Permission checking
    /// Check if a user has permission to perform an action on a tenant
    async fn check_tenant_permission(
        &self,
        user_arn: &str,
        tenant_id: &TenantId,
        action: TenantAction,
    ) -> Result<bool>;

    // Quota management
    /// Get effective quotas for a tenant (considering inheritance)
    async fn get_effective_quotas(&self, tenant_id: &TenantId) -> Result<TenantQuotas>;

    /// Get current resource usage for a tenant
    async fn get_tenant_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage>;
}
