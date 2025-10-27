#[cfg(test)]
mod sts_tests {
    use crate::store::memory::InMemoryStore;
    use crate::sts::{AssumeRoleRequest, GetSessionTokenRequest, StsClient};

    fn create_test_client() -> StsClient<InMemoryStore> {
        let store = InMemoryStore::new();
        StsClient::new(store)
    }

    #[tokio::test]
    async fn test_get_caller_identity() {
        let mut client = create_test_client();

        let response = client.get_caller_identity().await.unwrap();
        assert!(response.success);

        let identity = response.data.unwrap();
        assert!(!identity.account.is_empty());
        assert!(identity.arn.contains("arn:aws"));
        assert!(!identity.user_id.is_empty());
    }

    #[tokio::test]
    async fn test_assume_role() {
        let mut client = create_test_client();

        let request = AssumeRoleRequest {
            role_arn: "arn:aws:iam::123456789012:role/TestRole".to_string(),
            role_session_name: "test-session".to_string(),
            policy: None,
            duration_seconds: Some(3600),
            external_id: None,
        };

        let response = client.assume_role(request).await.unwrap();
        assert!(response.success);

        let result = response.data.unwrap();
        let credentials = result.credentials;
        assert!(!credentials.access_key_id.is_empty());
        assert!(!credentials.secret_access_key.is_empty());
        assert!(!credentials.session_token.is_empty());
        assert!(credentials.expiration > chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_assume_role_with_saml() {
        let mut client = create_test_client();

        let response = client
            .assume_role_with_saml(
                "arn:aws:iam::123456789012:role/SAMLRole".to_string(),
                "arn:aws:iam::123456789012:saml-provider/ExampleProvider".to_string(),
                "base64-encoded-saml-assertion".to_string(),
            )
            .await
            .unwrap();

        assert!(response.success);

        let result = response.data.unwrap();
        let credentials = result.credentials;
        assert!(!credentials.access_key_id.is_empty());
        assert!(!credentials.secret_access_key.is_empty());
    }

    #[tokio::test]
    async fn test_get_session_token() {
        let mut client = create_test_client();

        let request = GetSessionTokenRequest {
            duration_seconds: Some(1800),
            serial_number: None,
            token_code: None,
        };

        let response = client.get_session_token(Some(request)).await.unwrap();
        assert!(response.success);

        let credentials = response.data.unwrap();
        assert!(!credentials.access_key_id.is_empty());
        assert!(!credentials.secret_access_key.is_empty());
        assert!(!credentials.session_token.is_empty());

        // Check expiration is approximately correct (1800 seconds from now)
        let expected_expiration = chrono::Utc::now() + chrono::Duration::seconds(1800);
        let time_diff = (credentials.expiration - expected_expiration)
            .num_seconds()
            .abs();
        assert!(time_diff < 5, "Expiration time should be within 5 seconds");
    }

    #[tokio::test]
    async fn test_get_federation_token() {
        let mut client = create_test_client();

        let request = crate::sts::GetFederationTokenRequest {
            name: "federated-user".to_string(),
            policy: None,
            duration_seconds: Some(7200),
        };

        let response = client.get_federation_token(request).await.unwrap();

        assert!(response.success);

        let result = response.data.unwrap();
        let credentials = result.credentials;
        assert!(!credentials.access_key_id.is_empty());
        assert!(!credentials.secret_access_key.is_empty());
    }

    #[tokio::test]
    async fn test_decode_authorization_message() {
        let client = create_test_client();

        let encoded_message = "encoded-authorization-message";
        let response = client
            .decode_authorization_message(encoded_message.to_string())
            .await
            .unwrap();

        assert!(response.success);

        let decoded = response.data.unwrap();
        assert!(decoded.contains("Decoded message"));
    }

    #[tokio::test]
    async fn test_get_access_key_info() {
        let mut client = create_test_client();

        let response = client
            .get_access_key_info("AKIAIOSFODNN7EXAMPLE".to_string())
            .await
            .unwrap();

        assert!(response.success);

        let account_id = response.data.unwrap();
        assert!(!account_id.is_empty());
    }

    // Note: test_account_id_consistency removed - account_id is no longer stored at the store level
    // Resources now carry their own provider-specific information

    #[tokio::test]
    async fn test_credentials_have_valid_format() {
        let mut client = create_test_client();

        let request = AssumeRoleRequest {
            role_arn: "arn:aws:iam::123456789012:role/TestRole".to_string(),
            role_session_name: "format-test".to_string(),
            policy: None,
            duration_seconds: None,
            external_id: None,
        };

        let response = client.assume_role(request).await.unwrap();
        let result = response.data.unwrap();
        let credentials = result.credentials;

        // Access key should start with 'ASIA' for temporary credentials
        assert!(credentials.access_key_id.starts_with("ASIA"));
        // Access key should be at least 16 characters
        assert!(credentials.access_key_id.len() >= 16);

        // Secret access key should be at least 32 characters
        assert!(credentials.secret_access_key.len() >= 32);

        // Session token should exist and be non-empty
        assert!(!credentials.session_token.is_empty());
    }
}
