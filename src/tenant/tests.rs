//! Tests for Multi-Tenant Functionality

#[cfg(test)]
mod tenant_tests {
    use crate::store::memory::InMemoryStore;
    use crate::tenant::client::{CreateRootTenantRequest, CreateSubTenantRequest};
    use crate::tenant::{TenantClient, TenantId, TenantQuotas, TenantStatus, TenantType};
    use std::collections::HashMap;

    fn create_test_store() -> InMemoryStore {
        InMemoryStore::new()
    }

    #[tokio::test]
    async fn test_create_root_tenant() {
        let store = create_test_store();
        let mut client = TenantClient::new(store, "admin@example.com".to_string());

        let request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: HashMap::new(),
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@example.com".to_string()],
            metadata: HashMap::new(),
            billing_info: None,
        };

        let response = client.create_root_tenant(request).await.unwrap();
        let tenant = response.data.unwrap();

        assert_eq!(tenant.name, "acme");
        assert_eq!(tenant.id.as_str(), "acme");
        assert!(tenant.parent_id.is_none());
        assert_eq!(tenant.status, TenantStatus::Active);
        assert_eq!(tenant.max_child_depth, 5);
    }

    #[tokio::test]
    async fn test_create_sub_tenant() {
        let store = create_test_store();
        let mut client = TenantClient::new(store, "admin@example.com".to_string());

        // Create root tenant
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: HashMap::new(),
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@example.com".to_string()],
            metadata: HashMap::new(),
            billing_info: None,
        };

        client.create_root_tenant(root_request).await.unwrap();

        // Create child tenant
        let root_id = TenantId::root("acme");
        let child_request = CreateSubTenantRequest {
            name: "engineering".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: None, // Should inherit
            admin_principals: vec!["eng-admin@example.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        let response = client
            .create_sub_tenant(&root_id, child_request)
            .await
            .unwrap();
        let child = response.data.unwrap();

        assert_eq!(child.name, "engineering");
        assert_eq!(child.id.as_str(), "acme/engineering");
        assert_eq!(child.parent_id, Some(root_id.clone()));
        assert_eq!(child.id.depth(), 1);
    }

    #[tokio::test]
    async fn test_tenant_hierarchy() {
        let store = create_test_store();
        let mut client = TenantClient::new(store, "admin@example.com".to_string());

        // Create root
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: HashMap::new(),
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@example.com".to_string()],
            metadata: HashMap::new(),
            billing_info: None,
        };
        client.create_root_tenant(root_request).await.unwrap();

        // Create child
        let root_id = TenantId::root("acme");
        let child_request = CreateSubTenantRequest {
            name: "engineering".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: None,
            admin_principals: vec!["eng-admin@example.com".to_string()],
            metadata: None,
            billing_info: None,
        };
        client
            .create_sub_tenant(&root_id, child_request)
            .await
            .unwrap();

        // Create grandchild
        let child_id = root_id.child("engineering");
        let grandchild_request = CreateSubTenantRequest {
            name: "frontend".to_string(),
            organization: None,
            tenant_type: TenantType::Team,
            provider_accounts: None,
            quotas: None,
            admin_principals: vec!["frontend-lead@example.com".to_string()],
            metadata: None,
            billing_info: None,
        };
        client
            .create_sub_tenant(&child_id, grandchild_request)
            .await
            .unwrap();

        // Verify hierarchy
        let grandchild_id = child_id.child("frontend");
        assert_eq!(grandchild_id.as_str(), "acme/engineering/frontend");
        assert_eq!(grandchild_id.depth(), 2);
        assert!(grandchild_id.is_descendant_of(&root_id));
        assert!(grandchild_id.is_descendant_of(&child_id));

        // List children
        let children_response = client.list_child_tenants(&root_id).await.unwrap();
        let children = children_response.data.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "engineering");

        let grandchildren_response = client.list_child_tenants(&child_id).await.unwrap();
        let grandchildren = grandchildren_response.data.unwrap();
        assert_eq!(grandchildren.len(), 1);
        assert_eq!(grandchildren[0].name, "frontend");
    }

    #[tokio::test]
    async fn test_quota_enforcement() {
        let store = create_test_store();
        let mut client = TenantClient::new(store, "admin@example.com".to_string());

        // Create root with limited sub-tenants
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: HashMap::new(),
            quotas: Some(TenantQuotas {
                max_sub_tenants: 1, // Only 1 child allowed
                ..Default::default()
            }),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@example.com".to_string()],
            metadata: HashMap::new(),
            billing_info: None,
        };
        client.create_root_tenant(root_request).await.unwrap();

        let root_id = TenantId::root("acme");

        // First child should succeed
        let child1_request = CreateSubTenantRequest {
            name: "engineering".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: None,
            admin_principals: vec!["eng-admin@example.com".to_string()],
            metadata: None,
            billing_info: None,
        };
        client
            .create_sub_tenant(&root_id, child1_request)
            .await
            .unwrap();

        // Second child should fail (quota exceeded)
        let child2_request = CreateSubTenantRequest {
            name: "sales".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: None,
            admin_principals: vec!["sales-admin@example.com".to_string()],
            metadata: None,
            billing_info: None,
        };
        let result = client.create_sub_tenant(&root_id, child2_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quota_validation() {
        let store = create_test_store();
        let mut client = TenantClient::new(store, "admin@example.com".to_string());

        // Create root with specific quotas
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: HashMap::new(),
            quotas: Some(TenantQuotas {
                max_users: 100,
                max_roles: 50,
                ..Default::default()
            }),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@example.com".to_string()],
            metadata: HashMap::new(),
            billing_info: None,
        };
        client.create_root_tenant(root_request).await.unwrap();

        let root_id = TenantId::root("acme");

        // Try to create child with quotas exceeding parent
        let child_request = CreateSubTenantRequest {
            name: "engineering".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: Some(TenantQuotas {
                max_users: 200, // Exceeds parent quota
                max_roles: 50,
                ..Default::default()
            }),
            admin_principals: vec!["eng-admin@example.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        let result = client.create_sub_tenant(&root_id, child_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_tenant_usage() {
        let store = create_test_store();
        let mut client = TenantClient::new(store, "admin@example.com".to_string());

        // Create root
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: HashMap::new(),
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@example.com".to_string()],
            metadata: HashMap::new(),
            billing_info: None,
        };
        client.create_root_tenant(root_request).await.unwrap();

        let root_id = TenantId::root("acme");

        // Create child
        let child_request = CreateSubTenantRequest {
            name: "engineering".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: None,
            admin_principals: vec!["eng-admin@example.com".to_string()],
            metadata: None,
            billing_info: None,
        };
        client
            .create_sub_tenant(&root_id, child_request)
            .await
            .unwrap();

        // Get usage
        let usage_response = client.get_tenant_usage(&root_id).await.unwrap();
        let usage = usage_response.data.unwrap();

        assert_eq!(usage.tenant_id, root_id);
        assert_eq!(usage.current_sub_tenants, 1);
    }

    #[tokio::test]
    async fn test_delete_tenant_cascade() {
        let store = create_test_store();
        let mut client = TenantClient::new(store, "admin@example.com".to_string());

        // Create root
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: HashMap::new(),
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@example.com".to_string()],
            metadata: HashMap::new(),
            billing_info: None,
        };
        client.create_root_tenant(root_request).await.unwrap();

        let root_id = TenantId::root("acme");

        // Create child
        let child_request = CreateSubTenantRequest {
            name: "engineering".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: None,
            admin_principals: vec!["eng-admin@example.com".to_string()],
            metadata: None,
            billing_info: None,
        };
        client
            .create_sub_tenant(&root_id, child_request)
            .await
            .unwrap();

        // Delete root with cascade
        client.delete_tenant(&root_id, true).await.unwrap();

        // Verify both are deleted
        let root_result = client.get_tenant(&root_id).await;
        assert!(root_result.is_err()); // Should not exist

        let child_id = root_id.child("engineering");
        let child_result = client.get_tenant(&child_id).await;
        assert!(child_result.is_err()); // Should not exist
    }

    #[tokio::test]
    async fn test_tenant_id_operations() {
        let root = TenantId::root("acme");
        let child = root.child("engineering");
        let grandchild = child.child("frontend");

        // Test as_str
        assert_eq!(root.as_str(), "acme");
        assert_eq!(child.as_str(), "acme/engineering");
        assert_eq!(grandchild.as_str(), "acme/engineering/frontend");

        // Test depth
        assert_eq!(root.depth(), 0);
        assert_eq!(child.depth(), 1);
        assert_eq!(grandchild.depth(), 2);

        // Test parent
        assert_eq!(child.parent(), Some(root.clone()));
        assert_eq!(grandchild.parent(), Some(child.clone()));
        assert_eq!(root.parent(), None);

        // Test is_descendant_of
        assert!(child.is_descendant_of(&root));
        assert!(grandchild.is_descendant_of(&root));
        assert!(grandchild.is_descendant_of(&child));
        assert!(!root.is_descendant_of(&child));
        assert!(!child.is_descendant_of(&grandchild));

        // Test ancestors
        let ancestors = grandchild.ancestors();
        assert_eq!(ancestors.len(), 3);
        assert_eq!(ancestors[0], root);
        assert_eq!(ancestors[1], child);
        assert_eq!(ancestors[2], grandchild);
    }
}
