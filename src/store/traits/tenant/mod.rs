//! Tenant Store Trait

use crate::error::Result;
use crate::wami::tenant::{Tenant, TenantId, TenantQuotas, TenantUsage};
use async_trait::async_trait;

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

    // Quota management
    /// Get effective quotas for a tenant (considering inheritance)
    async fn get_effective_quotas(&self, tenant_id: &TenantId) -> Result<TenantQuotas>;

    /// Get current resource usage for a tenant
    async fn get_tenant_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage>;
}
