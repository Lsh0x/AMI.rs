//! In-Memory Tenant Store Implementation

use super::TenantStore;
use crate::error::{AmiError, Result};
use crate::tenant::{QuotaMode, Tenant, TenantAction, TenantId, TenantQuotas, TenantUsage};
use async_trait::async_trait;
use std::collections::HashMap;

/// In-memory implementation of tenant store
///
/// This stores all tenant data in memory using HashMaps.
/// Data is lost when the program exits.
#[derive(Debug, Clone)]
pub struct InMemoryTenantStore {
    tenants: HashMap<TenantId, Tenant>,
}

impl Default for InMemoryTenantStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryTenantStore {
    /// Create a new in-memory tenant store
    pub fn new() -> Self {
        Self {
            tenants: HashMap::new(),
        }
    }
}

#[async_trait]
impl TenantStore for InMemoryTenantStore {
    async fn create_tenant(&mut self, tenant: Tenant) -> Result<Tenant> {
        // Check if tenant already exists
        if self.tenants.contains_key(&tenant.id) {
            return Err(AmiError::ResourceExists {
                resource: format!("Tenant {} already exists", tenant.id),
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
        if self.tenants.remove(tenant_id).is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("Tenant {} not found", tenant_id),
            });
        }
        Ok(())
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        Ok(self.tenants.values().cloned().collect())
    }

    async fn list_child_tenants(&self, parent_id: &TenantId) -> Result<Vec<Tenant>> {
        let children: Vec<Tenant> = self
            .tenants
            .values()
            .filter(|t| t.parent_id.as_ref() == Some(parent_id))
            .cloned()
            .collect();
        Ok(children)
    }

    async fn get_ancestors(&self, tenant_id: &TenantId) -> Result<Vec<Tenant>> {
        let ancestor_ids = tenant_id.ancestors();
        let mut ancestors = Vec::new();

        for id in ancestor_ids {
            if let Some(tenant) = self.tenants.get(&id) {
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

    async fn check_tenant_permission(
        &self,
        user_arn: &str,
        tenant_id: &TenantId,
        _action: TenantAction,
    ) -> Result<bool> {
        // Simple permission check: user must be in admin_principals
        if let Some(tenant) = self.tenants.get(tenant_id) {
            if tenant.admin_principals.contains(&user_arn.to_string()) {
                return Ok(true);
            }

            // Check if user is admin in any ancestor tenant
            for ancestor_id in tenant_id.ancestors() {
                if let Some(ancestor) = self.tenants.get(&ancestor_id) {
                    if ancestor.admin_principals.contains(&user_arn.to_string()) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    async fn get_effective_quotas(&self, tenant_id: &TenantId) -> Result<TenantQuotas> {
        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Tenant {} not found", tenant_id),
            })?;

        // If tenant has override quotas, return them
        if matches!(tenant.quota_mode, QuotaMode::Override) {
            return Ok(tenant.quotas.clone());
        }

        // Otherwise, inherit from parent
        if let Some(parent_id) = &tenant.parent_id {
            return self.get_effective_quotas(parent_id).await;
        }

        // Root tenant with inherited mode - return its quotas
        Ok(tenant.quotas.clone())
    }

    async fn get_tenant_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage> {
        // Check if tenant exists
        if !self.tenants.contains_key(tenant_id) {
            return Err(AmiError::ResourceNotFound {
                resource: format!("Tenant {} not found", tenant_id),
            });
        }

        // Count sub-tenants
        let children = self.list_child_tenants(tenant_id).await?;
        let current_sub_tenants = children.len();

        // For now, return basic usage
        // In a real implementation, this would query IAM stores
        Ok(TenantUsage {
            tenant_id: tenant_id.clone(),
            current_users: 0,
            current_roles: 0,
            current_policies: 0,
            current_groups: 0,
            current_sub_tenants,
            include_descendants: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::{TenantStatus, TenantType};

    fn create_test_tenant(id: TenantId, parent_id: Option<TenantId>) -> Tenant {
        Tenant {
            id: id.clone(),
            parent_id,
            name: id.as_str().split('/').next_back().unwrap().to_string(),
            organization: None,
            tenant_type: TenantType::Enterprise,
            provider_accounts: HashMap::new(),
            created_at: chrono::Utc::now(),
            status: TenantStatus::Active,
            quotas: TenantQuotas::default(),
            quota_mode: QuotaMode::Inherited,
            max_child_depth: 5,
            can_create_sub_tenants: true,
            admin_principals: vec!["admin@example.com".to_string()],
            metadata: HashMap::new(),
            billing_info: None,
        }
    }

    #[tokio::test]
    async fn test_create_and_get_tenant() {
        let mut store = InMemoryTenantStore::new();
        let tenant_id = TenantId::root("acme");
        let tenant = create_test_tenant(tenant_id.clone(), None);

        let created = store.create_tenant(tenant).await.unwrap();
        assert_eq!(created.id, tenant_id);

        let retrieved = store.get_tenant(&tenant_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, tenant_id);
    }

    #[tokio::test]
    async fn test_list_child_tenants() {
        let mut store = InMemoryTenantStore::new();
        let root_id = TenantId::root("acme");
        let child1_id = root_id.child("engineering");
        let child2_id = root_id.child("sales");

        store
            .create_tenant(create_test_tenant(root_id.clone(), None))
            .await
            .unwrap();
        store
            .create_tenant(create_test_tenant(child1_id.clone(), Some(root_id.clone())))
            .await
            .unwrap();
        store
            .create_tenant(create_test_tenant(child2_id.clone(), Some(root_id.clone())))
            .await
            .unwrap();

        let children = store.list_child_tenants(&root_id).await.unwrap();
        assert_eq!(children.len(), 2);
    }

    #[tokio::test]
    async fn test_get_descendants() {
        let mut store = InMemoryTenantStore::new();
        let root_id = TenantId::root("acme");
        let child_id = root_id.child("engineering");
        let grandchild_id = child_id.child("frontend");

        store
            .create_tenant(create_test_tenant(root_id.clone(), None))
            .await
            .unwrap();
        store
            .create_tenant(create_test_tenant(child_id.clone(), Some(root_id.clone())))
            .await
            .unwrap();
        store
            .create_tenant(create_test_tenant(
                grandchild_id.clone(),
                Some(child_id.clone()),
            ))
            .await
            .unwrap();

        let descendants = store.get_descendants(&root_id).await.unwrap();
        assert_eq!(descendants.len(), 2); // child and grandchild
    }
}
