//! In-Memory Tenant Store Implementation

use crate::error::{AmiError, Result};
use crate::store::traits::TenantStore;
use crate::wami::tenant::{Tenant, TenantId, TenantQuotas, TenantUsage};
use async_trait::async_trait;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

/// In-memory implementation of tenant store
#[derive(Debug, Clone, Default)]
pub struct InMemoryTenantStore {
    tenants: HashMap<TenantId, Tenant>,
}

impl InMemoryTenantStore {
    /// Create a new empty tenant store
    pub fn new() -> Self {
        Self {
            tenants: HashMap::new(),
        }
    }
}

#[async_trait]
impl TenantStore for InMemoryTenantStore {
    async fn create_tenant(&mut self, tenant: Tenant) -> Result<Tenant> {
        if self.tenants.contains_key(&tenant.id) {
            return Err(AmiError::ResourceExists {
                resource: format!("Tenant {}", tenant.id),
            });
        }

        self.tenants.insert(tenant.id.clone(), tenant.clone());
        Ok(tenant)
    }

    async fn get_tenant(&self, tenant_id: &TenantId) -> Result<Option<Tenant>> {
        Ok(self.tenants.get(tenant_id).cloned())
    }

    async fn update_tenant(&mut self, tenant: Tenant) -> Result<Tenant> {
        if !self.tenants.contains_key(&tenant.id) {
            return Err(AmiError::ResourceNotFound {
                resource: format!("Tenant {} not found", tenant.id),
            });
        }

        self.tenants.insert(tenant.id.clone(), tenant.clone());
        Ok(tenant)
    }

    async fn delete_tenant(&mut self, tenant_id: &TenantId) -> Result<()> {
        self.tenants
            .remove(tenant_id)
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Tenant {} not found", tenant_id),
            })?;
        Ok(())
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        Ok(self.tenants.values().cloned().collect())
    }

    async fn list_child_tenants(&self, parent_id: &TenantId) -> Result<Vec<Tenant>> {
        Ok(self
            .tenants
            .values()
            .filter(|t| {
                t.parent_id
                    .as_ref()
                    .map(|p| p == parent_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }

    async fn get_ancestors(&self, tenant_id: &TenantId) -> Result<Vec<Tenant>> {
        let mut ancestors = Vec::new();
        let ancestor_ids = tenant_id.ancestors();

        // Exclude self - only include actual ancestors
        for id in ancestor_ids.iter().filter(|id| *id != tenant_id) {
            if let Some(tenant) = self.tenants.get(id) {
                ancestors.push(tenant.clone());
            }
        }

        Ok(ancestors)
    }

    async fn get_descendants(&self, tenant_id: &TenantId) -> Result<Vec<TenantId>> {
        let mut descendants = Vec::new();

        for id in self.tenants.keys() {
            if id.is_descendant_of(tenant_id) {
                descendants.push(id.clone());
            }
        }

        Ok(descendants)
    }

    async fn get_effective_quotas(&self, tenant_id: &TenantId) -> Result<TenantQuotas> {
        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Tenant {} not found", tenant_id),
            })?;

        Ok(tenant.quotas.clone())
    }

    async fn get_tenant_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage> {
        // This will be populated by the IAM store
        // For now, return empty usage
        Ok(TenantUsage {
            tenant_id: tenant_id.clone(),
            current_users: 0,
            current_roles: 0,
            current_policies: 0,
            current_groups: 0,
            current_sub_tenants: self.list_child_tenants(tenant_id).await?.len(),
            include_descendants: false,
        })
    }
}
