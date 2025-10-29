#[cfg(test)]
mod sso_tests {
    use crate::wami::sso_admin::{
        CreateAccountAssignmentRequest, CreatePermissionSetRequest, SsoAdminClient,
    };
    use crate::store::memory::InMemoryStore;

    fn create_test_client() -> SsoAdminClient<InMemoryStore> {
        let store = InMemoryStore::new();
        SsoAdminClient::new(store)
    }

    #[tokio::test]
    async fn test_create_permission_set() {
        let mut client = create_test_client();

        let request = CreatePermissionSetRequest {
            instance_arn: "arn:aws:sso:::instance/ssoins-1234567890abcdef".to_string(),
            name: "DeveloperAccess".to_string(),
            description: Some("Developer access permissions".to_string()),
            session_duration: Some("PT8H".to_string()),
            relay_state: None,
        };

        let response = client.create_permission_set(request).await.unwrap();
        assert!(response.success);

        let permission_set = response.data.unwrap();
        assert_eq!(permission_set.name, "DeveloperAccess");
        assert_eq!(permission_set.session_duration, Some("PT8H".to_string()));
        assert!(permission_set.permission_set_arn.contains("permissionSet"));
    }

    #[tokio::test]
    async fn test_describe_permission_set() {
        let mut client = create_test_client();

        // Create a permission set first
        let create_request = CreatePermissionSetRequest {
            instance_arn: "arn:aws:sso:::instance/ssoins-test".to_string(),
            name: "TestPermissionSet".to_string(),
            description: Some("Test description".to_string()),
            session_duration: None,
            relay_state: None,
        };
        let create_response = client.create_permission_set(create_request).await.unwrap();
        let permission_set_arn = create_response.data.unwrap().permission_set_arn;

        // Describe the permission set
        let response = client
            .describe_permission_set(
                "arn:aws:sso:::instance/ssoins-test".to_string(),
                permission_set_arn.clone(),
            )
            .await
            .unwrap();

        assert!(response.success);

        let permission_set = response.data.unwrap();
        assert_eq!(permission_set.name, "TestPermissionSet");
        assert_eq!(permission_set.permission_set_arn, permission_set_arn);
    }

    #[tokio::test]
    async fn test_delete_permission_set() {
        let mut client = create_test_client();

        // Create a permission set
        let create_request = CreatePermissionSetRequest {
            instance_arn: "arn:aws:sso:::instance/ssoins-delete-test".to_string(),
            name: "ToBeDeleted".to_string(),
            description: None,
            session_duration: None,
            relay_state: None,
        };
        let create_response = client.create_permission_set(create_request).await.unwrap();
        let permission_set_arn = create_response.data.unwrap().permission_set_arn;

        // Delete the permission set
        let response = client
            .delete_permission_set(
                "arn:aws:sso:::instance/ssoins-delete-test".to_string(),
                permission_set_arn.clone(),
            )
            .await
            .unwrap();

        assert!(response.success);

        // Verify it's deleted
        let result = client
            .describe_permission_set(
                "arn:aws:sso:::instance/ssoins-delete-test".to_string(),
                permission_set_arn,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_permission_sets() {
        let mut client = create_test_client();

        let instance_arn = "arn:aws:sso:::instance/ssoins-list-test".to_string();

        // Create multiple permission sets
        for i in 1..=3 {
            let request = CreatePermissionSetRequest {
                instance_arn: instance_arn.clone(),
                name: format!("PermissionSet{}", i),
                description: None,
                session_duration: None,
                relay_state: None,
            };
            client.create_permission_set(request).await.unwrap();
        }

        // List permission sets
        let response = client.list_permission_sets(instance_arn).await.unwrap();
        assert!(response.success);

        let permission_sets = response.data.unwrap();
        assert_eq!(permission_sets.len(), 3);
    }

    #[tokio::test]
    async fn test_create_account_assignment() {
        let mut client = create_test_client();

        let request = CreateAccountAssignmentRequest {
            instance_arn: "arn:aws:sso:::instance/ssoins-assignment".to_string(),
            target_id: "123456789012".to_string(),
            target_type: "AWS_ACCOUNT".to_string(),
            permission_set_arn: "arn:aws:sso:::permissionSet/ps-123".to_string(),
            principal_type: "USER".to_string(),
            principal_id: "user-123".to_string(),
        };

        let response = client.create_account_assignment(request).await.unwrap();
        assert!(response.success);

        let assignment = response.data.unwrap();
        assert_eq!(assignment.account_id, "123456789012");
        assert_eq!(assignment.principal_id, "user-123");
        assert_eq!(assignment.principal_type, "USER");
    }

    #[tokio::test]
    async fn test_delete_account_assignment() {
        let mut client = create_test_client();

        // Create an assignment first
        let create_request = CreateAccountAssignmentRequest {
            instance_arn: "arn:aws:sso:::instance/ssoins-delete-assignment".to_string(),
            target_id: "987654321098".to_string(),
            target_type: "AWS_ACCOUNT".to_string(),
            permission_set_arn: "arn:aws:sso:::permissionSet/ps-delete".to_string(),
            principal_type: "GROUP".to_string(),
            principal_id: "group-456".to_string(),
        };
        client
            .create_account_assignment(create_request)
            .await
            .unwrap();

        // Delete the assignment
        let response = client
            .delete_account_assignment(
                "arn:aws:sso:::instance/ssoins-delete-assignment".to_string(),
                "987654321098".to_string(),
                "AWS_ACCOUNT".to_string(),
                "arn:aws:sso:::permissionSet/ps-delete".to_string(),
                "GROUP".to_string(),
                "group-456".to_string(),
            )
            .await
            .unwrap();

        assert!(response.success);
    }

    #[tokio::test]
    async fn test_list_account_assignments() {
        let mut client = create_test_client();

        let instance_arn = "arn:aws:sso:::instance/ssoins-list-assignments".to_string();
        let account_id = "111111111111".to_string();
        let permission_set_arn = "arn:aws:sso:::permissionSet/ps-list".to_string();

        // Create multiple assignments
        for i in 1..=2 {
            let request = CreateAccountAssignmentRequest {
                instance_arn: instance_arn.clone(),
                target_id: account_id.clone(),
                target_type: "AWS_ACCOUNT".to_string(),
                permission_set_arn: permission_set_arn.clone(),
                principal_type: "USER".to_string(),
                principal_id: format!("user-{}", i),
            };
            client.create_account_assignment(request).await.unwrap();
        }

        // List assignments
        let response = client
            .list_account_assignments(instance_arn, account_id, permission_set_arn)
            .await
            .unwrap();

        assert!(response.success);

        let assignments = response.data.unwrap();
        assert_eq!(assignments.len(), 2);
    }

    #[tokio::test]
    async fn test_list_instances() {
        let mut client = create_test_client();

        let response = client.list_instances().await.unwrap();
        assert!(response.success);

        // Instances list should be returned (may be empty in mock implementation)
        let _instances = response.data.unwrap();
        // Note: In the mock implementation, this might be empty or contain default instances
    }

    #[tokio::test]
    async fn test_create_trusted_token_issuer() {
        let mut client = create_test_client();

        let response = client
            .create_trusted_token_issuer(
                "arn:aws:sso:::instance/ssoins-issuer".to_string(),
                "MyIssuer".to_string(),
                "https://issuer.example.com".to_string(),
            )
            .await
            .unwrap();

        assert!(response.success);

        let issuer = response.data.unwrap();
        assert_eq!(issuer.name, "MyIssuer");
        assert_eq!(issuer.issuer_url, "https://issuer.example.com");
        assert!(issuer
            .trusted_token_issuer_arn
            .contains("trustedTokenIssuer"));
    }

    #[tokio::test]
    async fn test_list_trusted_token_issuers() {
        let mut client = create_test_client();

        let instance_arn = "arn:aws:sso:::instance/ssoins-list-issuers".to_string();

        // Create an issuer
        client
            .create_trusted_token_issuer(
                instance_arn.clone(),
                "Issuer1".to_string(),
                "https://issuer1.example.com".to_string(),
            )
            .await
            .unwrap();

        // List issuers
        let response = client
            .list_trusted_token_issuers(instance_arn)
            .await
            .unwrap();

        assert!(response.success);

        let issuers = response.data.unwrap();
        assert!(!issuers.is_empty());
    }
}
