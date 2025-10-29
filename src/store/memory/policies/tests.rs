//! Tests for Policy Store Implementation

use crate::provider::aws::AwsProvider;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::PolicyStore;
use crate::types::PaginationParams;
use crate::wami::policies::policy::builder as policy_builder;

#[tokio::test]
async fn test_policy_create_and_get() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let policy_doc = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":"s3:*","Resource":"*"}]}"#.to_string();
    let policy = policy_builder::build_policy(
        "S3FullAccess".to_string(),
        policy_doc,
        Some("/".to_string()),
        Some("Full S3 access policy".to_string()),
        None, // tags
        &provider,
        "123456789012",
    );

    let policy_arn = policy.arn.clone();

    // Create policy
    let created = store.create_policy(policy.clone()).await.unwrap();
    assert_eq!(created.policy_name, "S3FullAccess");

    // Get policy
    let retrieved = store.get_policy(&policy_arn).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().policy_name, "S3FullAccess");
}

#[tokio::test]
async fn test_policy_get_nonexistent() {
    let store = InMemoryWamiStore::new();

    let result = store
        .get_policy("arn:aws:iam::123456789012:policy/nonexistent")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_policy_update() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let policy_doc = r#"{"Version":"2012-10-17"}"#.to_string();
    let policy = policy_builder::build_policy(
        "TestPolicy".to_string(),
        policy_doc,
        Some("/".to_string()),
        Some("Original description".to_string()),
        None, // tags
        &provider,
        "123456789012",
    );

    let policy_arn = policy.arn.clone();
    store.create_policy(policy.clone()).await.unwrap();

    // Update description
    let mut updated = policy;
    updated.description = Some("Updated description".to_string());
    store.update_policy(updated).await.unwrap();

    // Verify update
    let retrieved = store.get_policy(&policy_arn).await.unwrap().unwrap();
    assert_eq!(
        retrieved.description,
        Some("Updated description".to_string())
    );
}

#[tokio::test]
async fn test_policy_delete() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let policy_doc = r#"{"Version":"2012-10-17"}"#.to_string();
    let policy = policy_builder::build_policy(
        "TempPolicy".to_string(),
        policy_doc,
        Some("/".to_string()),
        None, // description
        None, // tags
        &provider,
        "123456789012",
    );

    let policy_arn = policy.arn.clone();
    store.create_policy(policy).await.unwrap();

    // Delete policy
    store.delete_policy(&policy_arn).await.unwrap();

    // Verify deleted
    let result = store.get_policy(&policy_arn).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_policy_list_empty() {
    let store = InMemoryWamiStore::new();

    let (policies, is_truncated, marker) = store.list_policies(None, None).await.unwrap();

    assert_eq!(policies.len(), 0);
    assert!(!is_truncated);
    assert!(marker.is_none());
}

#[tokio::test]
async fn test_policy_list_multiple() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let policy_doc = r#"{"Version":"2012-10-17"}"#.to_string();

    for name in &["PolicyA", "PolicyB", "PolicyC"] {
        let policy = policy_builder::build_policy(
            name.to_string(),
            policy_doc.clone(),
            Some("/".to_string()),
            None, // description
            None, // tags
            &provider,
            "123456789012",
        );
        store.create_policy(policy).await.unwrap();
    }

    let (policies, is_truncated, _) = store.list_policies(None, None).await.unwrap();

    assert_eq!(policies.len(), 3);
    assert!(!is_truncated);
}

#[tokio::test]
async fn test_policy_list_with_path_prefix() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let policy_doc = r#"{"Version":"2012-10-17"}"#.to_string();

    let policy1 = policy_builder::build_policy(
        "AdminPolicy".to_string(),
        policy_doc.clone(),
        Some("/admin/".to_string()),
        None, // description
        None, // tags
        &provider,
        "123456789012",
    );
    let policy2 = policy_builder::build_policy(
        "UserPolicy".to_string(),
        policy_doc,
        Some("/users/".to_string()),
        None, // description
        None, // tags
        &provider,
        "123456789012",
    );

    store.create_policy(policy1).await.unwrap();
    store.create_policy(policy2).await.unwrap();

    let (policies, _, _) = store.list_policies(Some("/admin/"), None).await.unwrap();

    assert_eq!(policies.len(), 1);
    assert!(policies[0].path.starts_with("/admin/"));
}

#[tokio::test]
async fn test_policy_list_with_pagination() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let policy_doc = r#"{"Version":"2012-10-17"}"#.to_string();

    // Create 5 policies
    for i in 0..5 {
        let policy = policy_builder::build_policy(
            format!("Policy{}", i),
            policy_doc.clone(),
            Some("/".to_string()),
            None, // description
            None, // tags
            &provider,
            "123456789012",
        );
        store.create_policy(policy).await.unwrap();
    }

    let pagination = PaginationParams {
        max_items: Some(3),
        marker: None,
    };

    let (policies, is_truncated, marker) =
        store.list_policies(None, Some(&pagination)).await.unwrap();

    assert_eq!(policies.len(), 3);
    assert!(is_truncated);
    assert!(marker.is_some());
}
