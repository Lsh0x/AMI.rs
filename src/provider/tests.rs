//! Comprehensive integration tests for the provider system
//!
//! These tests verify that all providers work correctly with IAM operations
//! and that resource limits and validations are enforced.

#[cfg(test)]
mod integration_tests {
    use crate::iam::IamClient;
    use crate::provider::{
        aws::AwsProvider, azure::AzureProvider, custom::CustomProvider, gcp::GcpProvider,
        CloudProvider, ResourceLimits, ResourceType,
    };
    use crate::store::in_memory::InMemoryStore;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_aws_provider_with_user_creation() {
        use crate::iam::users::CreateUserRequest;

        let provider = Arc::new(AwsProvider::default());
        let store = InMemoryStore::with_account_and_provider("123456789012".to_string(), provider);
        let mut client = IamClient::new(store);

        let request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            tags: None,
            permissions_boundary: None,
        };

        let response = client.create_user(request).await.unwrap();
        assert!(response.success);

        let user = response.data.unwrap();
        assert_eq!(user.user_name, "alice");
        assert!(user.user_id.starts_with("AIDA")); // AWS user ID prefix
        assert!(user.arn.contains("arn:aws:iam::123456789012:user/alice"));
    }

    #[tokio::test]
    async fn test_gcp_provider_with_user_creation() {
        use crate::iam::users::CreateUserRequest;

        let provider = Arc::new(GcpProvider::new("my-project-123"));
        let store =
            InMemoryStore::with_account_and_provider("my-project-123".to_string(), provider);
        let mut client = IamClient::new(store);

        let request = CreateUserRequest {
            user_name: "bob".to_string(),
            path: None,
            tags: None,
            permissions_boundary: None,
        };

        let response = client.create_user(request).await.unwrap();
        assert!(response.success);

        let user = response.data.unwrap();
        assert_eq!(user.user_name, "bob");
        // GCP uses numeric IDs
        assert!(user.user_id.parse::<u128>().is_ok());
        // GCP uses service account format
        assert!(user
            .arn
            .contains("projects/my-project-123/serviceAccounts/bob"));
    }

    #[tokio::test]
    async fn test_azure_provider_with_user_creation() {
        use crate::iam::users::CreateUserRequest;

        let provider = Arc::new(AzureProvider::new("sub-123", "rg-prod"));
        let store = InMemoryStore::with_account_and_provider("sub-123".to_string(), provider);
        let mut client = IamClient::new(store);

        let request = CreateUserRequest {
            user_name: "charlie".to_string(),
            path: None,
            tags: None,
            permissions_boundary: None,
        };

        let response = client.create_user(request).await.unwrap();
        assert!(response.success);

        let user = response.data.unwrap();
        assert_eq!(user.user_name, "charlie");
        // Azure uses GUIDs
        assert!(uuid::Uuid::parse_str(&user.user_id).is_ok());
        // Azure resource ID format
        assert!(user.arn.contains("/subscriptions/sub-123/"));
        assert!(user.arn.contains("/resourceGroups/rg-prod/"));
    }

    #[tokio::test]
    async fn test_custom_provider_with_user_creation() {
        use crate::iam::users::CreateUserRequest;

        let provider = Arc::new(
            CustomProvider::builder()
                .name("mycloud")
                .arn_template("mycloud://{account}/user/{name}")
                .id_prefix("MYC")
                .build(),
        );
        let store = InMemoryStore::with_account_and_provider("tenant-42".to_string(), provider);
        let mut client = IamClient::new(store);

        let request = CreateUserRequest {
            user_name: "dana".to_string(),
            path: None,
            tags: None,
            permissions_boundary: None,
        };

        let response = client.create_user(request).await.unwrap();
        assert!(response.success);

        let user = response.data.unwrap();
        assert_eq!(user.user_name, "dana");
        assert!(user.user_id.starts_with("MYC")); // Custom prefix
        assert_eq!(user.arn, "mycloud://tenant-42/user/dana");
    }

    #[tokio::test]
    async fn test_access_key_limit_enforcement_aws() {
        use crate::iam::{access_keys::CreateAccessKeyRequest, users::CreateUserRequest};

        let provider = Arc::new(AwsProvider::default());
        let store = InMemoryStore::with_account_and_provider("123456789012".to_string(), provider);
        let mut client = IamClient::new(store);

        // Create user first
        client
            .create_user(CreateUserRequest {
                user_name: "limituser".to_string(),
                path: None,
                tags: None,
                permissions_boundary: None,
            })
            .await
            .unwrap();

        // AWS limit is 2 access keys per user
        let request = CreateAccessKeyRequest {
            user_name: "limituser".to_string(),
        };

        // First key - should succeed
        let response1 = client.create_access_key(request.clone()).await;
        assert!(response1.is_ok());

        // Second key - should succeed
        let response2 = client.create_access_key(request.clone()).await;
        assert!(response2.is_ok());

        // Third key - should fail due to AWS limit
        let response3 = client.create_access_key(request.clone()).await;
        assert!(response3.is_err());
    }

    #[tokio::test]
    async fn test_access_key_limit_enforcement_gcp() {
        use crate::iam::{access_keys::CreateAccessKeyRequest, users::CreateUserRequest};

        let provider = Arc::new(GcpProvider::new("my-project"));
        let store = InMemoryStore::with_account_and_provider("my-project".to_string(), provider);
        let mut client = IamClient::new(store);

        // Create user first
        client
            .create_user(CreateUserRequest {
                user_name: "gcplimituser".to_string(),
                path: None,
                tags: None,
                permissions_boundary: None,
            })
            .await
            .unwrap();

        // GCP limit is 10 access keys per user
        let request = CreateAccessKeyRequest {
            user_name: "gcplimituser".to_string(),
        };

        // Create 10 keys - all should succeed
        for i in 0..10 {
            let response = client.create_access_key(request.clone()).await;
            assert!(response.is_ok(), "Key {} should succeed", i + 1);
        }

        // 11th key - should fail due to GCP limit
        let response11 = client.create_access_key(request.clone()).await;
        assert!(response11.is_err());
    }

    #[tokio::test]
    async fn test_custom_provider_with_custom_limits() {
        use crate::iam::{access_keys::CreateAccessKeyRequest, users::CreateUserRequest};

        let custom_limits = ResourceLimits {
            max_access_keys_per_user: 5,
            max_service_credentials_per_user_per_service: 3,
            max_tags_per_resource: 100,
            session_duration_min: 1800, // 30 minutes
            session_duration_max: 7200, // 2 hours
            max_mfa_devices_per_user: 5,
            max_signing_certificates_per_user: 2,
        };

        let provider = Arc::new(
            CustomProvider::builder()
                .name("restrictive-cloud")
                .id_prefix("RC")
                .limits(custom_limits)
                .build(),
        );

        let store =
            InMemoryStore::with_account_and_provider("tenant-1".to_string(), provider.clone());
        let mut client = IamClient::new(store);

        // Create user
        client
            .create_user(CreateUserRequest {
                user_name: "customuser".to_string(),
                path: None,
                tags: None,
                permissions_boundary: None,
            })
            .await
            .unwrap();

        // Test custom limit of 5 access keys
        let request = CreateAccessKeyRequest {
            user_name: "customuser".to_string(),
        };

        // Create 5 keys - all should succeed
        for i in 0..5 {
            let response = client.create_access_key(request.clone()).await;
            assert!(
                response.is_ok(),
                "Key {} should succeed with custom limit",
                i + 1
            );
        }

        // 6th key - should fail due to custom limit
        let response6 = client.create_access_key(request.clone()).await;
        assert!(
            response6.is_err(),
            "6th key should fail with custom limit of 5"
        );

        // Verify limits are correctly set
        let limits = provider.resource_limits();
        assert_eq!(limits.max_access_keys_per_user, 5);
        assert_eq!(limits.max_tags_per_resource, 100);
        assert_eq!(limits.session_duration_max, 7200);
    }

    #[tokio::test]
    async fn test_provider_name_consistency() {
        let aws = AwsProvider::default();
        let gcp = GcpProvider::new("test-project");
        let azure = AzureProvider::new("test-sub", "test-rg");
        let custom = CustomProvider::builder().name("mycloud").build();

        assert_eq!(aws.name(), "aws");
        assert_eq!(gcp.name(), "gcp");
        assert_eq!(azure.name(), "azure");
        assert_eq!(custom.name(), "mycloud");
    }

    #[test]
    fn test_all_resource_types_have_ids() {
        let aws = AwsProvider::default();
        let resource_types = vec![
            ResourceType::User,
            ResourceType::Group,
            ResourceType::Role,
            ResourceType::Policy,
            ResourceType::AccessKey,
            ResourceType::ServerCertificate,
            ResourceType::ServiceCredential,
            ResourceType::ServiceLinkedRole,
            ResourceType::SigningCertificate,
            ResourceType::MfaDevice,
        ];

        for resource_type in resource_types {
            let id = aws.generate_resource_id(resource_type);
            assert!(
                !id.is_empty(),
                "ID should not be empty for {:?}",
                resource_type
            );

            // AWS IDs should have 4-letter prefix + 17 chars = 21 total
            assert_eq!(
                id.len(),
                21,
                "AWS ID length incorrect for {:?}",
                resource_type
            );
        }
    }

    #[test]
    fn test_aws_id_prefixes() {
        let aws = AwsProvider::default();

        assert!(aws
            .generate_resource_id(ResourceType::User)
            .starts_with("AIDA"));
        assert!(aws
            .generate_resource_id(ResourceType::Group)
            .starts_with("AGPA"));
        assert!(aws
            .generate_resource_id(ResourceType::Role)
            .starts_with("AROA"));
        assert!(aws
            .generate_resource_id(ResourceType::Policy)
            .starts_with("ANPA"));
        assert!(aws
            .generate_resource_id(ResourceType::AccessKey)
            .starts_with("AKIA"));
        assert!(aws
            .generate_resource_id(ResourceType::ServerCertificate)
            .starts_with("ASCA"));
        assert!(aws
            .generate_resource_id(ResourceType::ServiceCredential)
            .starts_with("ACCA"));
        assert!(aws
            .generate_resource_id(ResourceType::ServiceLinkedRole)
            .starts_with("AROA"));
        assert!(aws
            .generate_resource_id(ResourceType::SigningCertificate)
            .starts_with("ASCA"));
    }

    #[test]
    fn test_provider_switch_compatibility() {
        // Ensure all providers implement the same trait methods
        let aws = Arc::new(AwsProvider::default()) as Arc<dyn CloudProvider>;
        let gcp = Arc::new(GcpProvider::new("test")) as Arc<dyn CloudProvider>;
        let azure = Arc::new(AzureProvider::new("s", "r")) as Arc<dyn CloudProvider>;
        let custom = Arc::new(CustomProvider::builder().build()) as Arc<dyn CloudProvider>;

        // All should be able to generate IDs
        for provider in [aws, gcp, azure, custom] {
            let id = provider.generate_resource_id(ResourceType::User);
            assert!(!id.is_empty());
        }
    }
}
