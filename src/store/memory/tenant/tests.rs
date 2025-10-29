//! Tests for Tenant Store Implementation

use crate::error::AmiError;
use crate::store::memory::tenant::InMemoryTenantStore;
use crate::store::traits::TenantStore;
use crate::wami::tenant::{QuotaMode, Tenant, TenantId, TenantQuotas, TenantStatus, TenantType};

fn build_test_tenant(name: &str, parent: Option<TenantId>) -> Tenant {
    let tenant_id = TenantId::new(name);
    Tenant {
        id: tenant_id.clone(),
        name: name.to_string(),
        parent_id: parent,
        organization: Some(format!("{} Organization", name)),
        status: TenantStatus::Active,
        tenant_type: TenantType::Enterprise,
        provider_accounts: std::collections::HashMap::new(),
        arn: format!("arn:wami:tenant::{}", name),
        providers: Vec::new(),
        created_at: chrono::Utc::now(),
        quotas: TenantQuotas::default(),
        quota_mode: QuotaMode::Inherited,
        max_child_depth: 5,
        can_create_sub_tenants: true,
        admin_principals: Vec::new(),
        metadata: std::collections::HashMap::new(),
        billing_info: None,
    }
}

#[tokio::test]
async fn test_tenant_create_and_get() {
    let mut store = InMemoryTenantStore::new();

    let tenant = build_test_tenant("acme-corp", None);
    let tenant_id = tenant.id.clone();

    // Create tenant
    let created = store.create_tenant(tenant.clone()).await.unwrap();
    assert_eq!(created.name, "acme-corp");

    // Get tenant
    let retrieved = store.get_tenant(&tenant_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "acme-corp");
}

#[tokio::test]
async fn test_tenant_get_nonexistent() {
    let store = InMemoryTenantStore::new();

    let result = store
        .get_tenant(&TenantId::new("nonexistent"))
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_tenant_create_duplicate_fails() {
    let mut store = InMemoryTenantStore::new();

    let tenant = build_test_tenant("acme-corp", None);

    store.create_tenant(tenant.clone()).await.unwrap();

    // Try to create duplicate
    let result = store.create_tenant(tenant).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AmiError::ResourceExists { .. }
    ));
}

#[tokio::test]
async fn test_tenant_update() {
    let mut store = InMemoryTenantStore::new();

    let tenant = build_test_tenant("acme-corp", None);
    let tenant_id = tenant.id.clone();

    store.create_tenant(tenant.clone()).await.unwrap();

    // Update tenant
    let mut updated = tenant;
    updated.status = TenantStatus::Suspended;
    store.update_tenant(updated).await.unwrap();

    // Verify update
    let retrieved = store.get_tenant(&tenant_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, TenantStatus::Suspended);
}

#[tokio::test]
async fn test_tenant_update_nonexistent_fails() {
    let mut store = InMemoryTenantStore::new();

    let tenant = build_test_tenant("nonexistent", None);

    let result = store.update_tenant(tenant).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AmiError::ResourceNotFound { .. }
    ));
}

#[tokio::test]
async fn test_tenant_delete() {
    let mut store = InMemoryTenantStore::new();

    let tenant = build_test_tenant("temp-tenant", None);
    let tenant_id = tenant.id.clone();

    store.create_tenant(tenant).await.unwrap();

    // Delete tenant
    store.delete_tenant(&tenant_id).await.unwrap();

    // Verify deleted
    let result = store.get_tenant(&tenant_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_tenant_delete_nonexistent_fails() {
    let mut store = InMemoryTenantStore::new();

    let result = store.delete_tenant(&TenantId::new("nonexistent")).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AmiError::ResourceNotFound { .. }
    ));
}

#[tokio::test]
async fn test_tenant_list_empty() {
    let store = InMemoryTenantStore::new();

    let tenants = store.list_tenants().await.unwrap();
    assert_eq!(tenants.len(), 0);
}

#[tokio::test]
async fn test_tenant_list_multiple() {
    let mut store = InMemoryTenantStore::new();

    // Create multiple tenants
    for name in &["tenant-a", "tenant-b", "tenant-c"] {
        let tenant = build_test_tenant(name, None);
        store.create_tenant(tenant).await.unwrap();
    }

    let tenants = store.list_tenants().await.unwrap();
    assert_eq!(tenants.len(), 3);
}

#[tokio::test]
async fn test_tenant_hierarchy() {
    let mut store = InMemoryTenantStore::new();

    // Create parent
    let parent = build_test_tenant("parent", None);
    let parent_id = parent.id.clone();
    store.create_tenant(parent).await.unwrap();

    // Create children
    let child1 = build_test_tenant("parent.child1", Some(parent_id.clone()));
    let child2 = build_test_tenant("parent.child2", Some(parent_id.clone()));
    store.create_tenant(child1).await.unwrap();
    store.create_tenant(child2).await.unwrap();

    // List children
    let children = store.list_child_tenants(&parent_id).await.unwrap();
    assert_eq!(children.len(), 2);
}

#[tokio::test]
async fn test_tenant_list_child_tenants_empty() {
    let mut store = InMemoryTenantStore::new();

    let tenant = build_test_tenant("lonely-tenant", None);
    let tenant_id = tenant.id.clone();
    store.create_tenant(tenant).await.unwrap();

    let children = store.list_child_tenants(&tenant_id).await.unwrap();
    assert_eq!(children.len(), 0);
}

#[tokio::test]
async fn test_tenant_get_ancestors() {
    let mut store = InMemoryTenantStore::new();

    // Create hierarchy: root -> child -> grandchild (using proper TenantId hierarchy)
    let root_id = TenantId::root("root");
    let mut root = build_test_tenant("root", None);
    root.id = root_id.clone();
    store.create_tenant(root).await.unwrap();

    let child_id = root_id.child("child");
    let mut child = build_test_tenant("child", Some(root_id.clone()));
    child.id = child_id.clone();
    store.create_tenant(child).await.unwrap();

    let grandchild_id = child_id.child("grandchild");
    let mut grandchild = build_test_tenant("grandchild", Some(child_id.clone()));
    grandchild.id = grandchild_id.clone();
    store.create_tenant(grandchild).await.unwrap();

    // Get ancestors of grandchild
    let ancestors = store.get_ancestors(&grandchild_id).await.unwrap();
    assert_eq!(ancestors.len(), 2); // Should include root and child
}

#[tokio::test]
async fn test_tenant_get_ancestors_root() {
    let mut store = InMemoryTenantStore::new();

    let root_id = TenantId::root("root");
    let mut root = build_test_tenant("root", None);
    root.id = root_id.clone();
    store.create_tenant(root).await.unwrap();

    // Root tenant should have no ancestors
    let ancestors = store.get_ancestors(&root_id).await.unwrap();
    assert_eq!(ancestors.len(), 0);
}

#[tokio::test]
async fn test_tenant_get_descendants() {
    let mut store = InMemoryTenantStore::new();

    // Create hierarchy (using proper TenantId hierarchy)
    let root_id = TenantId::root("root");
    let mut root = build_test_tenant("root", None);
    root.id = root_id.clone();
    store.create_tenant(root).await.unwrap();

    let child1_id = root_id.child("child1");
    let mut child1 = build_test_tenant("child1", Some(root_id.clone()));
    child1.id = child1_id.clone();
    store.create_tenant(child1).await.unwrap();

    let child2_id = root_id.child("child2");
    let mut child2 = build_test_tenant("child2", Some(root_id.clone()));
    child2.id = child2_id.clone();
    store.create_tenant(child2).await.unwrap();

    let grandchild_id = child1_id.child("grandchild");
    let mut grandchild = build_test_tenant("grandchild", Some(child1_id));
    grandchild.id = grandchild_id.clone();
    store.create_tenant(grandchild).await.unwrap();

    // Get descendants of root
    let descendants = store.get_descendants(&root_id).await.unwrap();
    assert_eq!(descendants.len(), 3); // child1, child2, grandchild
}

#[tokio::test]
async fn test_tenant_get_descendants_leaf() {
    let mut store = InMemoryTenantStore::new();

    let tenant = build_test_tenant("leaf", None);
    let tenant_id = tenant.id.clone();
    store.create_tenant(tenant).await.unwrap();

    // Leaf tenant should have no descendants
    let descendants = store.get_descendants(&tenant_id).await.unwrap();
    assert_eq!(descendants.len(), 0);
}

#[tokio::test]
async fn test_tenant_get_effective_quotas() {
    let mut store = InMemoryTenantStore::new();

    let mut tenant = build_test_tenant("acme-corp", None);
    tenant.quotas = TenantQuotas {
        max_users: 100,
        max_roles: 50,
        max_policies: 200,
        max_groups: 75,
        max_access_keys: 500,
        max_sub_tenants: 10,
        api_rate_limit: 1000,
    };
    tenant.quota_mode = QuotaMode::Override;

    let tenant_id = tenant.id.clone();
    store.create_tenant(tenant).await.unwrap();

    let quotas = store.get_effective_quotas(&tenant_id).await.unwrap();
    assert_eq!(quotas.max_users, 100);
    assert_eq!(quotas.max_roles, 50);
}

#[tokio::test]
async fn test_tenant_get_effective_quotas_nonexistent() {
    let store = InMemoryTenantStore::new();

    let result = store
        .get_effective_quotas(&TenantId::new("nonexistent"))
        .await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AmiError::ResourceNotFound { .. }
    ));
}

#[tokio::test]
async fn test_tenant_get_usage() {
    let mut store = InMemoryTenantStore::new();

    let parent = build_test_tenant("parent", None);
    let parent_id = parent.id.clone();
    store.create_tenant(parent).await.unwrap();

    // Create children
    let child1 = build_test_tenant("parent.child1", Some(parent_id.clone()));
    let child2 = build_test_tenant("parent.child2", Some(parent_id.clone()));
    store.create_tenant(child1).await.unwrap();
    store.create_tenant(child2).await.unwrap();

    let usage = store.get_tenant_usage(&parent_id).await.unwrap();
    assert_eq!(usage.tenant_id, parent_id);
    assert_eq!(usage.current_sub_tenants, 2);
}

#[tokio::test]
async fn test_tenant_different_statuses() {
    let mut store = InMemoryTenantStore::new();

    let active = {
        let mut t = build_test_tenant("active-tenant", None);
        t.status = TenantStatus::Active;
        t
    };
    let suspended = {
        let mut t = build_test_tenant("suspended-tenant", None);
        t.status = TenantStatus::Suspended;
        t
    };

    store.create_tenant(active.clone()).await.unwrap();
    store.create_tenant(suspended.clone()).await.unwrap();

    let retrieved_active = store.get_tenant(&active.id).await.unwrap().unwrap();
    assert_eq!(retrieved_active.status, TenantStatus::Active);

    let retrieved_suspended = store.get_tenant(&suspended.id).await.unwrap().unwrap();
    assert_eq!(retrieved_suspended.status, TenantStatus::Suspended);
}

#[tokio::test]
async fn test_tenant_different_types() {
    let mut store = InMemoryTenantStore::new();

    let department = {
        let mut t = build_test_tenant("department", None);
        t.tenant_type = TenantType::Department;
        t
    };
    let project = {
        let mut t = build_test_tenant("project", None);
        t.tenant_type = TenantType::Project;
        t
    };

    store.create_tenant(department.clone()).await.unwrap();
    store.create_tenant(project.clone()).await.unwrap();

    let retrieved_dept = store.get_tenant(&department.id).await.unwrap().unwrap();
    assert_eq!(retrieved_dept.tenant_type, TenantType::Department);

    let retrieved_proj = store.get_tenant(&project.id).await.unwrap().unwrap();
    assert_eq!(retrieved_proj.tenant_type, TenantType::Project);
}
