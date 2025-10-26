//! Tenant Module Tests

#[cfg(test)]
mod tenant_tests {
    use crate::store::memory::InMemoryStore;
    use crate::tenant::client::{CreateRootTenantRequest, CreateSubTenantRequest};
    use crate::tenant::{TenantClient, TenantId, TenantQuotas, TenantStatus, TenantType};

    #[tokio::test]
    async fn test_create_root_tenant() {
        let store = InMemoryStore::new();
        let mut client = TenantClient::new(store, "admin@example.com".to_string());

        let request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: None,
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@acme.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        let response = client.create_root_tenant(request).await.unwrap();
        let tenant = response.data.unwrap();

        assert_eq!(tenant.id, TenantId::root("acme"));
        assert_eq!(tenant.name, "acme");
        assert_eq!(tenant.status, TenantStatus::Active);
        assert_eq!(tenant.parent_id, None);
    }

    #[tokio::test]
    async fn test_create_sub_tenant() {
        let store = InMemoryStore::new();
        // Use the same admin principal that will be set on tenants
        let mut client = TenantClient::new(store, "admin@acme.com".to_string());

        // Create root
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: Some("Acme Corp".to_string()),
            provider_accounts: None,
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@acme.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        client.create_root_tenant(root_request).await.unwrap();

        // Create sub-tenant
        let root_id = TenantId::root("acme");
        let sub_request = CreateSubTenantRequest {
            name: "engineering".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: None, // Inherit from parent
            admin_principals: vec!["eng-admin@acme.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        let response = client
            .create_sub_tenant(&root_id, sub_request)
            .await
            .unwrap();
        let sub_tenant = response.data.unwrap();

        assert_eq!(sub_tenant.id, TenantId::new("acme/engineering"));
        assert_eq!(sub_tenant.parent_id, Some(root_id));
        assert_eq!(sub_tenant.name, "engineering");
    }

    #[tokio::test]
    async fn test_tenant_hierarchy() {
        let id = TenantId::new("acme/engineering/team1");

        assert_eq!(id.depth(), 2);
        assert_eq!(id.parent().unwrap(), TenantId::new("acme/engineering"));

        let ancestors = id.ancestors();
        assert_eq!(ancestors.len(), 3);
        assert_eq!(ancestors[0], TenantId::new("acme"));
        assert_eq!(ancestors[1], TenantId::new("acme/engineering"));
        assert_eq!(ancestors[2], TenantId::new("acme/engineering/team1"));

        assert!(id.is_descendant_of(&TenantId::new("acme")));
        assert!(id.is_descendant_of(&TenantId::new("acme/engineering")));
        assert!(!id.is_descendant_of(&TenantId::new("other")));
    }

    #[tokio::test]
    async fn test_list_child_tenants() {
        let store = InMemoryStore::new();
        let mut client = TenantClient::new(store, "admin@acme.com".to_string());

        // Create root
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: None,
            provider_accounts: None,
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@acme.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        client.create_root_tenant(root_request).await.unwrap();

        // Create multiple children
        let root_id = TenantId::root("acme");

        for name in &["eng", "sales", "marketing"] {
            let sub_request = CreateSubTenantRequest {
                name: name.to_string(),
                organization: None,
                tenant_type: TenantType::Department,
                provider_accounts: None,
                quotas: None,
                admin_principals: vec![format!("admin-{}@acme.com", name)],
                metadata: None,
                billing_info: None,
            };
            client
                .create_sub_tenant(&root_id, sub_request)
                .await
                .unwrap();
        }

        // List children
        let response = client.list_child_tenants(&root_id).await.unwrap();
        let children = response.data.unwrap();

        assert_eq!(children.len(), 3);
        let names: Vec<String> = children.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"eng".to_string()));
        assert!(names.contains(&"sales".to_string()));
        assert!(names.contains(&"marketing".to_string()));
    }

    #[tokio::test]
    async fn test_tenant_quota_validation() {
        let parent_quotas = TenantQuotas {
            max_users: 100,
            max_roles: 50,
            max_policies: 20,
            max_groups: 10,
            max_access_keys: 200,
            max_sub_tenants: 5,
            api_rate_limit: 500,
        };

        let valid_child_quotas = TenantQuotas {
            max_users: 50,
            max_roles: 25,
            max_policies: 10,
            max_groups: 5,
            max_access_keys: 100,
            max_sub_tenants: 3,
            api_rate_limit: 250,
        };

        // Should pass
        assert!(valid_child_quotas
            .validate_against_parent(&parent_quotas)
            .is_ok());

        let invalid_child_quotas = TenantQuotas {
            max_users: 200, // Exceeds parent
            max_roles: 25,
            max_policies: 10,
            max_groups: 5,
            max_access_keys: 100,
            max_sub_tenants: 3,
            api_rate_limit: 250,
        };

        // Should fail
        assert!(invalid_child_quotas
            .validate_against_parent(&parent_quotas)
            .is_err());
    }

    #[tokio::test]
    async fn test_tenant_aware_user_path() {
        use crate::iam::user::builder::build_user;
        use crate::provider::AwsProvider;
        use std::sync::Arc;

        let provider = Arc::new(AwsProvider::new());
        let tenant_id = Some(TenantId::new("acme/engineering"));

        let user = build_user(
            "alice".to_string(),
            None,
            None,
            None,
            provider.as_ref(),
            "123456789012",
            tenant_id.clone(),
        );

        // Verify tenant-aware path
        assert_eq!(user.path, "/tenants/acme/engineering/");
        assert_eq!(user.tenant_id, tenant_id);
        assert!(user.arn.contains("/tenants/acme/engineering/alice"));
    }

    #[tokio::test]
    async fn test_delete_tenant_cascade() {
        let store = InMemoryStore::new();
        let mut client = TenantClient::new(store, "admin@acme.com".to_string());

        // Create root
        let root_request = CreateRootTenantRequest {
            name: "acme".to_string(),
            organization: None,
            provider_accounts: None,
            quotas: Some(TenantQuotas::default()),
            max_child_depth: Some(5),
            admin_principals: vec!["admin@acme.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        client.create_root_tenant(root_request).await.unwrap();

        // Create sub-tenant
        let root_id = TenantId::root("acme");
        let sub_request = CreateSubTenantRequest {
            name: "engineering".to_string(),
            organization: None,
            tenant_type: TenantType::Department,
            provider_accounts: None,
            quotas: None,
            admin_principals: vec!["eng-admin@acme.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        client
            .create_sub_tenant(&root_id, sub_request)
            .await
            .unwrap();

        // Create grandchild
        let eng_id = TenantId::new("acme/engineering");
        let grandchild_request = CreateSubTenantRequest {
            name: "team1".to_string(),
            organization: None,
            tenant_type: TenantType::Team,
            provider_accounts: None,
            quotas: None,
            admin_principals: vec!["team1-admin@acme.com".to_string()],
            metadata: None,
            billing_info: None,
        };

        client
            .create_sub_tenant(&eng_id, grandchild_request)
            .await
            .unwrap();

        // Delete cascade from engineering
        client.delete_tenant_cascade(&eng_id).await.unwrap();

        // Verify engineering and team1 are deleted
        let eng_result = client.get_tenant(&eng_id).await;
        assert!(eng_result.is_err());

        let team1_id = TenantId::new("acme/engineering/team1");
        let team1_result = client.get_tenant(&team1_id).await;
        assert!(team1_result.is_err());

        // Verify root still exists
        let root_result = client.get_tenant(&root_id).await;
        assert!(root_result.is_ok());
    }
}
