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

    /// Generate a unique tenant ID with global uniqueness validation
    ///
    /// This function generates a numeric tenant ID and ensures it doesn't collide
    /// with any existing tenant (root or child) in the system.
    async fn generate_unique_tenant_id(&self, parent_id: Option<&TenantId>) -> Result<TenantId> {
        const MAX_RETRIES: usize = 10; // Extremely unlikely to need retries

        for _ in 0..MAX_RETRIES {
            let tenant_id = if let Some(parent) = parent_id {
                parent.child()
            } else {
                TenantId::root()
            };

            // Check global uniqueness - verify tenant doesn't already exist
            let exists = self.store.read().unwrap().get_tenant(&tenant_id).await?;

            if exists.is_none() {
                return Ok(tenant_id);
            }

            // Collision detected, retry (extremely rare with u64)
        }

        Err(crate::error::AmiError::ResourceLimitExceeded {
            resource_type: "tenant_id_generation".to_string(),
            limit: MAX_RETRIES,
        })
    }

    /// Validate that tenant name is unique within the parent
    ///
    /// Names must be unique within a parent tenant to enable name-to-ID mapping for UI display.
    async fn validate_name_uniqueness(
        &self,
        name: &str,
        parent_id: Option<&TenantId>,
    ) -> Result<()> {
        let children = if let Some(parent) = parent_id {
            self.list_child_tenants(parent).await?
        } else {
            // For root tenants, check all tenants without a parent
            self.list_tenants()
                .await?
                .into_iter()
                .filter(|t| t.parent_id.is_none())
                .collect()
        };

        // Check if name already exists within the parent
        if children.iter().any(|t| t.name == name) {
            return Err(crate::error::AmiError::ResourceExists {
                resource: format!(
                    "Tenant with name '{}' already exists{}",
                    name,
                    parent_id
                        .map(|p| format!(" in parent {}", p.as_str()))
                        .unwrap_or_else(|| " at root level".to_string())
                ),
            });
        }

        Ok(())
    }

    /// Find a tenant by name within a parent
    ///
    /// This enables name-to-ID mapping for UI display purposes.
    /// Names are unique within a parent, so this lookup is deterministic.
    ///
    /// # Arguments
    ///
    /// * `name` - The tenant name to search for
    /// * `parent_id` - The parent tenant ID (None for root-level tenants)
    ///
    /// # Returns
    ///
    /// The tenant if found, None otherwise
    pub async fn find_tenant_by_name(
        &self,
        name: &str,
        parent_id: Option<&TenantId>,
    ) -> Result<Option<Tenant>> {
        let candidates = if let Some(parent) = parent_id {
            self.list_child_tenants(parent).await?
        } else {
            // For root tenants, get all tenants without a parent
            self.list_tenants()
                .await?
                .into_iter()
                .filter(|t| t.parent_id.is_none())
                .collect()
        };

        Ok(candidates.into_iter().find(|t| t.name == name))
    }

    /// Create a new tenant
    pub async fn create_tenant(
        &self,
        context: &WamiContext,
        name: String,
        organization: Option<String>,
        parent_id: Option<TenantId>,
    ) -> Result<Tenant> {
        // Validate name format
        tenant_operations::validate_name(&name)?;

        // Validate name uniqueness within parent
        self.validate_name_uniqueness(&name, parent_id.as_ref())
            .await?;

        // Generate unique numeric tenant ID (globally validated)
        let tenant_id = self.generate_unique_tenant_id(parent_id.as_ref()).await?;

        // Build tenant using pure function with pre-generated ID
        let mut tenant = tenant_operations::build_tenant(tenant_id, name, organization, parent_id);

        // Generate ARN using context
        tenant.arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant_path(context.tenant_path().clone())
            .wami_instance(context.instance_id())
            .resource("tenant", tenant.id.as_str())
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
        let arn: WamiArn = "arn:wami:iam:12345678:wami:123456789012:user/test"
            .parse()
            .unwrap();
        crate::context::WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(12345678))
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

    #[tokio::test]
    async fn test_find_tenant_by_name_root_level() {
        let service = setup_service();
        let context = test_context();

        // Create multiple root tenants
        let tenant1 = service
            .create_tenant(&context, "acme".to_string(), None, None)
            .await
            .unwrap();
        service
            .create_tenant(&context, "globex".to_string(), None, None)
            .await
            .unwrap();

        // Find by name at root level
        let found = service.find_tenant_by_name("acme", None).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, tenant1.id);

        // Find non-existent
        let not_found = service
            .find_tenant_by_name("nonexistent", None)
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_find_tenant_by_name_with_parent() {
        let service = setup_service();
        let context = test_context();

        // Create parent
        let parent = service
            .create_tenant(&context, "parent".to_string(), None, None)
            .await
            .unwrap();

        // Create children with same name but different parent
        let child1 = service
            .create_tenant(
                &context,
                "engineering".to_string(),
                None,
                Some(parent.id.clone()),
            )
            .await
            .unwrap();

        // Create another parent with child of same name
        let parent2 = service
            .create_tenant(&context, "parent2".to_string(), None, None)
            .await
            .unwrap();
        service
            .create_tenant(
                &context,
                "engineering".to_string(),
                None,
                Some(parent2.id.clone()),
            )
            .await
            .unwrap();

        // Find child by name within specific parent
        let found = service
            .find_tenant_by_name("engineering", Some(&parent.id))
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, child1.id);
    }

    #[tokio::test]
    async fn test_name_uniqueness_validation() {
        let service = setup_service();
        let context = test_context();

        // Create tenant with name "test"
        service
            .create_tenant(&context, "test".to_string(), None, None)
            .await
            .unwrap();

        // Try to create another with same name at root level - should fail
        let result = service
            .create_tenant(&context, "test".to_string(), None, None)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_name_uniqueness_within_parent() {
        let service = setup_service();
        let context = test_context();

        // Create parent
        let parent = service
            .create_tenant(&context, "parent".to_string(), None, None)
            .await
            .unwrap();

        // Create child with name "eng"
        service
            .create_tenant(&context, "eng".to_string(), None, Some(parent.id.clone()))
            .await
            .unwrap();

        // Try to create another child with same name in same parent - should fail
        let result = service
            .create_tenant(&context, "eng".to_string(), None, Some(parent.id.clone()))
            .await;
        assert!(result.is_err());

        // But can create with same name in different parent
        let parent2 = service
            .create_tenant(&context, "parent2".to_string(), None, None)
            .await
            .unwrap();
        let result2 = service
            .create_tenant(&context, "eng".to_string(), None, Some(parent2.id.clone()))
            .await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_numeric_tenant_id_format() {
        let service = setup_service();
        let context = test_context();

        let tenant = service
            .create_tenant(&context, "test".to_string(), None, None)
            .await
            .unwrap();

        // Tenant ID should be numeric (slash-separated)
        let id_str = tenant.id.as_str();
        // Should parse as u64 or multiple u64s separated by /
        let parts: Vec<&str> = id_str.split('/').collect();
        for part in parts {
            assert!(
                part.parse::<u64>().is_ok(),
                "Invalid numeric segment: {}",
                part
            );
        }
    }

    #[tokio::test]
    async fn test_tenant_id_global_uniqueness() {
        let service = setup_service();
        let context = test_context();

        // Create multiple tenants - each should have unique numeric ID
        let tenant1 = service
            .create_tenant(&context, "tenant1".to_string(), None, None)
            .await
            .unwrap();
        let tenant2 = service
            .create_tenant(&context, "tenant2".to_string(), None, None)
            .await
            .unwrap();

        // IDs should be different (even though both are root tenants)
        assert_ne!(tenant1.id, tenant2.id);
    }
}
