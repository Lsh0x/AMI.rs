//! Tenant Store Trait and Implementations

pub mod memory;

use crate::error::Result;
use async_trait::async_trait;

use super::{Tenant, TenantAction, TenantId, TenantQuotas, TenantUsage};

pub use memory::InMemoryTenantStore;

/// Trait for tenant data storage operations
///
/// This trait defines the interface that any tenant storage backend must implement.
#[async_trait]
pub trait TenantStore: Send + Sync {
    // Basic CRUD
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
    /// List direct children of a tenant
    async fn list_child_tenants(&self, parent_id: &TenantId) -> Result<Vec<Tenant>>;

    /// Get all ancestors of a tenant (parent, grandparent, etc.)
    async fn get_ancestors(&self, tenant_id: &TenantId) -> Result<Vec<Tenant>>;

    /// Get all descendant tenant IDs (children, grandchildren, etc.)
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
    /// Get effective quotas for a tenant (with inheritance)
    async fn get_effective_quotas(&self, tenant_id: &TenantId) -> Result<TenantQuotas>;

    /// Get current resource usage for a tenant
    async fn get_tenant_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage>;
}
