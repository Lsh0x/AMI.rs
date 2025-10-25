use rustyiam::{
    create_memory_store, AssumeRoleRequest, CreateAccountAssignmentRequest,
    CreatePermissionSetRequest, CreateUserRequest, MemoryIamClient, MemorySsoAdminClient,
    MemoryStsClient,
};

#[tokio::test]
async fn test_end_to_end_workflow() {
    // Initialize all clients with a shared store
    let store = create_memory_store();
    let mut iam_client = MemoryIamClient::new(store.clone());
    let mut sts_client = MemoryStsClient::new(store.clone());
    let mut sso_client = MemorySsoAdminClient::new(store);

    // Step 1: Create an IAM user
    let user_request = CreateUserRequest {
        user_name: "integration-test-user".to_string(),
        path: Some("/test/".to_string()),
        permissions_boundary: None,
        tags: None,
    };

    let user_response = iam_client.create_user(user_request).await.unwrap();
    assert!(user_response.success);
    let user = user_response.data.unwrap();
    assert_eq!(user.user_name, "integration-test-user");

    // Step 2: Get caller identity from STS
    let identity_response = sts_client.get_caller_identity().await.unwrap();
    assert!(identity_response.success);
    let identity = identity_response.data.unwrap();
    assert!(!identity.account.is_empty());

    // Step 3: Assume a role
    let assume_role_request = AssumeRoleRequest {
        role_arn: "arn:aws:iam::123456789012:role/TestRole".to_string(),
        role_session_name: "integration-test-session".to_string(),
        policy: None,
        duration_seconds: Some(3600),
        external_id: None,
    };

    let credentials_response = sts_client.assume_role(assume_role_request).await.unwrap();
    assert!(credentials_response.success);
    let credentials = credentials_response.data.unwrap();
    assert!(!credentials.access_key_id.is_empty());
    assert!(!credentials.secret_access_key.is_empty());

    // Step 4: Create an SSO permission set
    let permission_set_request = CreatePermissionSetRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-integration-test".to_string(),
        name: "IntegrationTestPermissionSet".to_string(),
        description: Some("Integration test permission set".to_string()),
        session_duration: Some("PT8H".to_string()),
        relay_state: None,
    };

    let permission_set_response = sso_client
        .create_permission_set(permission_set_request)
        .await
        .unwrap();
    assert!(permission_set_response.success);
    let permission_set = permission_set_response.data.unwrap();

    // Step 5: Create an account assignment
    let assignment_request = CreateAccountAssignmentRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-integration-test".to_string(),
        target_id: identity.account.clone(),
        target_type: "AWS_ACCOUNT".to_string(),
        permission_set_arn: permission_set.permission_set_arn,
        principal_type: "USER".to_string(),
        principal_id: user.user_id,
    };

    let assignment_response = sso_client
        .create_account_assignment(assignment_request)
        .await
        .unwrap();
    assert!(assignment_response.success);
    let assignment = assignment_response.data.unwrap();
    assert_eq!(assignment.account_id, identity.account);

    // Step 6: Clean up - delete the user
    let delete_response = iam_client
        .delete_user("integration-test-user".to_string())
        .await
        .unwrap();
    assert!(delete_response.success);
}

#[tokio::test]
async fn test_multiple_users_workflow() {
    let store = create_memory_store();
    let mut iam_client = MemoryIamClient::new(store);

    // Create multiple users
    let user_names = vec!["alice", "bob", "charlie"];
    for name in &user_names {
        let request = CreateUserRequest {
            user_name: name.to_string(),
            path: Some("/team/engineering/".to_string()),
            permissions_boundary: None,
            tags: None,
        };
        let response = iam_client.create_user(request).await.unwrap();
        assert!(response.success);
    }

    // List users and verify
    let list_response = iam_client.list_users(None).await.unwrap();
    assert!(list_response.success);
    let list_result = list_response.data.unwrap();
    assert_eq!(list_result.users.len(), 3);

    // Clean up - delete all users
    for name in &user_names {
        let response = iam_client.delete_user(name.to_string()).await.unwrap();
        assert!(response.success);
    }

    // Verify all users are deleted
    let list_response = iam_client.list_users(None).await.unwrap();
    let list_result = list_response.data.unwrap();
    assert_eq!(list_result.users.len(), 0);
}

#[tokio::test]
async fn test_account_id_consistency() {
    let store = create_memory_store();
    let account_id = rustyiam::get_account_id_from_store(&store);

    let mut iam_client = MemoryIamClient::new(store.clone());
    let mut sts_client = MemoryStsClient::new(store.clone());

    // Create a user and check ARN contains the account ID
    let request = CreateUserRequest {
        user_name: "test-consistency".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    };
    let user_response = iam_client.create_user(request).await.unwrap();
    let user = user_response.data.unwrap();
    assert!(user.arn.contains(account_id));

    // Get caller identity and check account ID matches
    let identity_response = sts_client.get_caller_identity().await.unwrap();
    let identity = identity_response.data.unwrap();
    assert_eq!(identity.account, account_id);
}

#[tokio::test]
async fn test_error_handling() {
    let store = create_memory_store();
    let mut iam_client = MemoryIamClient::new(store);

    // Try to get a nonexistent user - should return error
    let result = iam_client.get_user("nonexistent-user".to_string()).await;
    assert!(
        result.is_err(),
        "Getting nonexistent user should return an error"
    );

    // Try to update a nonexistent user - should return error
    let update_request = rustyiam::UpdateUserRequest {
        user_name: "nonexistent-user".to_string(),
        new_user_name: None,
        new_path: Some("/new/".to_string()),
    };
    let result = iam_client.update_user(update_request).await;
    assert!(
        result.is_err(),
        "Updating nonexistent user should return an error"
    );
}

#[tokio::test]
async fn test_concurrent_operations() {
    use tokio::task;

    // Create multiple users concurrently using the same store
    let store = create_memory_store();
    let mut iam_client = MemoryIamClient::new(store.clone());

    let mut handles = vec![];
    for i in 0..5 {
        let mut client_clone = MemoryIamClient::new(store.clone());
        let handle = task::spawn(async move {
            let request = CreateUserRequest {
                user_name: format!("concurrent-user-{}", i),
                path: None,
                permissions_boundary: None,
                tags: None,
            };
            client_clone.create_user(request).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "User creation should succeed");
    }

    // Verify the operation completed successfully without panicking
    let list_response = iam_client.list_users(None).await.unwrap();
    assert!(list_response.success, "Should complete without errors");
}
