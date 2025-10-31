//! Tenant Management Service
//!
//! Orchestrates tenant operations.

use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::TenantStore;
use crate::wami::tenant::operations::tenant_operations;
use crate::wami::tenant::{Tenant, TenantId, TenantQuotas, TenantUsage};
use std::sync::{Arc, RwLock};

/// Service for managing tenants
///
/// Provides high-level operations for multi-tenant management.
pub struct TenantService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: TenantStore> TenantService<S> {
    /// Create a new TenantService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Create a new tenant
    pub async fn create_tenant(
        &self,
        context: &WamiContext,
        name: String,
        organization: Option<String>,
        parent_id: Option<TenantId>,
    ) -> Result<Tenant> {
        // Validate name
        tenant_operations::validate_name(&name)?;

        // Build tenant using pure function
        let mut tenant = tenant_operations::build_tenant(name, organization, parent_id);

        // Generate ARN using context
        tenant.arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant_path(context.tenant_path().clone())
            .wami_instance(context.instance_id())
            .resource("tenant", tenant.id.to_string())
            .build()?
            .to_string();

        // Persist
        self.store.write().unwrap().create_tenant(tenant).await
    }

    /// Get a tenant by ID
    pub async fn get_tenant(&self, tenant_id: &TenantId) -> Result<Option<Tenant>> {
        self.store.read().unwrap().get_tenant(tenant_id).await
    }

    /// Update a tenant
    pub async fn update_tenant(&self, tenant: Tenant) -> Result<Tenant> {
        self.store.write().unwrap().update_tenant(tenant).await
    }

    /// Delete a tenant
    pub async fn delete_tenant(&self, tenant_id: &TenantId) -> Result<()> {
        self.store.write().unwrap().delete_tenant(tenant_id).await
    }

    /// List all tenants
    pub async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        self.store.read().unwrap().list_tenants().await
    }

    /// List child tenants of a parent
    pub async fn list_child_tenants(&self, parent_id: &TenantId) -> Result<Vec<Tenant>> {
        self.store
            .read()
            .unwrap()
            .list_child_tenants(parent_id)
            .await
    }

    /// Get all ancestors of a tenant
    pub async fn get_ancestors(&self, tenant_id: &TenantId) -> Result<Vec<Tenant>> {
        self.store.read().unwrap().get_ancestors(tenant_id).await
    }

    /// Get all descendants of a tenant
    pub async fn get_descendants(&self, tenant_id: &TenantId) -> Result<Vec<TenantId>> {
        self.store.read().unwrap().get_descendants(tenant_id).await
    }

    /// Get effective quotas for a tenant (considering hierarchy)
    pub async fn get_effective_quotas(&self, tenant_id: &TenantId) -> Result<TenantQuotas> {
        self.store
            .read()
            .unwrap()
            .get_effective_quotas(tenant_id)
            .await
    }

    /// Get current usage for a tenant
    pub async fn get_tenant_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage> {
        self.store.read().unwrap().get_tenant_usage(tenant_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> TenantService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        TenantService::new(store)
    }

    fn test_context() -> crate::context::WamiContext {
        use crate::arn::{TenantPath, WamiArn};
        let arn: WamiArn = "arn:wami:iam:test:wami:123456789012:user/test"
            .parse()
            .unwrap();
        crate::context::WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("test"))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get_tenant() {
        let service = setup_service();

        let context = test_context();
        let tenant = service
            .create_tenant(
                &context,
                "acme-corp".to_string(),
                Some("ACME Inc".to_string()),
                None,
            )
            .await
            .unwrap();

        assert_eq!(tenant.name, "acme-corp");
        assert_eq!(tenant.organization, Some("ACME Inc".to_string()));
        assert!(!tenant.arn.is_empty());

        let retrieved = service.get_tenant(&tenant.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "acme-corp");
    }

    #[tokio::test]
    async fn test_list_tenants() {
        let service = setup_service();

        let context = test_context();
        service
            .create_tenant(&context, "tenant1".to_string(), None, None)
            .await
            .unwrap();
        service
            .create_tenant(&context, "tenant2".to_string(), None, None)
            .await
            .unwrap();

        let tenants = service.list_tenants().await.unwrap();
        assert_eq!(tenants.len(), 2);
    }

    #[tokio::test]
    async fn test_update_tenant() {
        let service = setup_service();

        let context = test_context();
        let mut tenant = service
            .create_tenant(&context, "test-tenant".to_string(), None, None)
            .await
            .unwrap();

        tenant.organization = Some("Updated Org".to_string());

        let updated = service.update_tenant(tenant).await.unwrap();
        assert_eq!(updated.organization, Some("Updated Org".to_string()));
    }

    #[tokio::test]
    async fn test_delete_tenant() {
        let service = setup_service();

        let context = test_context();
        let tenant = service
            .create_tenant(&context, "delete-me".to_string(), None, None)
            .await
            .unwrap();

        service.delete_tenant(&tenant.id).await.unwrap();

        let retrieved = service.get_tenant(&tenant.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_hierarchical_tenants() {
        let service = setup_service();

        let context = test_context();
        // Create parent
        let parent = service
            .create_tenant(&context, "parent".to_string(), None, None)
            .await
            .unwrap();

        // Create child
        let child = service
            .create_tenant(&context, "child".to_string(), None, Some(parent.id.clone()))
            .await
            .unwrap();

        // List children
        let children = service.list_child_tenants(&parent.id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "child");

        // Get ancestors
        let ancestors = service.get_ancestors(&child.id).await.unwrap();
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0].name, "parent");
    }

    #[tokio::test]
    async fn test_get_descendants() {
        let service = setup_service();

        let context = test_context();
        let root = service
            .create_tenant(&context, "root".to_string(), None, None)
            .await
            .unwrap();

        let child1 = service
            .create_tenant(&context, "child1".to_string(), None, Some(root.id.clone()))
            .await
            .unwrap();

        service
            .create_tenant(&context, "child2".to_string(), None, Some(root.id.clone()))
            .await
            .unwrap();

        service
            .create_tenant(
                &context,
                "grandchild".to_string(),
                None,
                Some(child1.id.clone()),
            )
            .await
            .unwrap();

        let descendants = service.get_descendants(&root.id).await.unwrap();
        assert_eq!(descendants.len(), 3); // child1, child2, grandchild
    }

    #[tokio::test]
    async fn test_validate_invalid_name() {
        let service = setup_service();

        let context = test_context();
        let result = service
            .create_tenant(&context, "".to_string(), None, None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_provider() {
        let service = setup_service();
        let context = test_context();

        let tenant = service
            .create_tenant(&context, "gcp-tenant".to_string(), None, None)
            .await
            .unwrap();

        // ARN should reflect WAMI format
        assert!(tenant.arn.contains("wami"));
    }
}
