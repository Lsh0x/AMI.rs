//! Tests for Identity Store Implementations
//!
//! Tests for UserStore, GroupStore, RoleStore, and ServiceLinkedRoleStore

use crate::provider::aws::AwsProvider;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::{GroupStore, RoleStore, ServiceLinkedRoleStore, UserStore};
use crate::types::{PaginationParams, Tag};
use crate::wami::identity::group::builder as group_builder;
use crate::wami::identity::role::builder as role_builder;
use crate::wami::identity::service_linked_role::builder as slr_builder;
use crate::wami::identity::user::builder as user_builder;

// ============================================================================
// USER STORE TESTS
// ============================================================================

#[tokio::test]
async fn test_user_create_and_get() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let user = user_builder::build_user(
        "alice".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012",
    );

    // Create user
    let created = store.create_user(user.clone()).await.unwrap();
    assert_eq!(created.user_name, "alice");

    // Get user
    let retrieved = store.get_user("alice").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().user_name, "alice");
}

#[tokio::test]
async fn test_user_get_nonexistent() {
    let store = InMemoryWamiStore::new();

    let result = store.get_user("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_user_update() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let user = user_builder::build_user(
        "bob".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012",
    );

    store.create_user(user.clone()).await.unwrap();

    // Update user
    let updated = user_builder::update_user_path(user, "/admin/".to_string());
    let result = store.update_user(updated).await.unwrap();

    assert_eq!(result.path, "/admin/");

    // Verify update persisted
    let retrieved = store.get_user("bob").await.unwrap().unwrap();
    assert_eq!(retrieved.path, "/admin/");
}

#[tokio::test]
async fn test_user_delete() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let user = user_builder::build_user(
        "charlie".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012",
    );

    store.create_user(user).await.unwrap();

    // Delete user
    store.delete_user("charlie").await.unwrap();

    // Verify deleted
    let result = store.get_user("charlie").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_user_list_empty() {
    let store = InMemoryWamiStore::new();

    let (users, is_truncated, marker) = store.list_users(None, None).await.unwrap();

    assert_eq!(users.len(), 0);
    assert!(!is_truncated);
    assert!(marker.is_none());
}

#[tokio::test]
async fn test_user_list_multiple() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    // Create multiple users
    for name in &["alice", "bob", "charlie", "david"] {
        let user = user_builder::build_user(
            name.to_string(),
            Some("/".to_string()),
            &provider,
            "123456789012",
        );
        store.create_user(user).await.unwrap();
    }

    let (users, is_truncated, _) = store.list_users(None, None).await.unwrap();

    assert_eq!(users.len(), 4);
    assert!(!is_truncated);
    // Should be sorted by name
    assert_eq!(users[0].user_name, "alice");
    assert_eq!(users[3].user_name, "david");
}

#[tokio::test]
async fn test_user_list_with_path_prefix() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let user1 = user_builder::build_user(
        "admin1".to_string(),
        Some("/admin/".to_string()),
        &provider,
        "123456789012",
    );
    let user2 = user_builder::build_user(
        "user1".to_string(),
        Some("/users/".to_string()),
        &provider,
        "123456789012",
    );
    let user3 = user_builder::build_user(
        "admin2".to_string(),
        Some("/admin/".to_string()),
        &provider,
        "123456789012",
    );

    store.create_user(user1).await.unwrap();
    store.create_user(user2).await.unwrap();
    store.create_user(user3).await.unwrap();

    let (users, _, _) = store.list_users(Some("/admin/"), None).await.unwrap();

    assert_eq!(users.len(), 2);
    assert!(users.iter().all(|u| u.path.starts_with("/admin/")));
}

#[tokio::test]
async fn test_user_list_with_pagination() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    // Create 10 users
    for i in 0..10 {
        let user = user_builder::build_user(
            format!("user{:02}", i),
            Some("/".to_string()),
            &provider,
            "123456789012",
        );
        store.create_user(user).await.unwrap();
    }

    let pagination = PaginationParams {
        max_items: Some(5),
        marker: None,
    };

    let (users, is_truncated, marker) = store.list_users(None, Some(&pagination)).await.unwrap();

    assert_eq!(users.len(), 5);
    assert!(is_truncated);
    assert!(marker.is_some());
}

#[tokio::test]
async fn test_user_tag_operations() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let user = user_builder::build_user(
        "alice".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012",
    );

    store.create_user(user).await.unwrap();

    // Add tags
    let tags = vec![
        Tag {
            key: "Environment".to_string(),
            value: "Production".to_string(),
        },
        Tag {
            key: "Team".to_string(),
            value: "Backend".to_string(),
        },
    ];

    store.tag_user("alice", tags).await.unwrap();

    // List tags
    let retrieved_tags = store.list_user_tags("alice").await.unwrap();
    assert_eq!(retrieved_tags.len(), 2);

    // Untag
    store
        .untag_user("alice", vec!["Team".to_string()])
        .await
        .unwrap();

    let remaining_tags = store.list_user_tags("alice").await.unwrap();
    assert_eq!(remaining_tags.len(), 1);
    assert_eq!(remaining_tags[0].key, "Environment");
}

// ============================================================================
// GROUP STORE TESTS
// ============================================================================

#[tokio::test]
async fn test_group_create_and_get() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let group = group_builder::build_group(
        "admins".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012",
    );

    let created = store.create_group(group.clone()).await.unwrap();
    assert_eq!(created.group_name, "admins");

    let retrieved = store.get_group("admins").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().group_name, "admins");
}

#[tokio::test]
async fn test_group_delete() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let group = group_builder::build_group(
        "devs".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012",
    );

    store.create_group(group).await.unwrap();
    store.delete_group("devs").await.unwrap();

    let result = store.get_group("devs").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_group_list() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    for name in &["admins", "devs", "ops"] {
        let group = group_builder::build_group(
            name.to_string(),
            Some("/".to_string()),
            &provider,
            "123456789012",
        );
        store.create_group(group).await.unwrap();
    }

    let (groups, is_truncated, _) = store.list_groups(None, None).await.unwrap();

    assert_eq!(groups.len(), 3);
    assert!(!is_truncated);
}

#[tokio::test]
async fn test_group_user_membership() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let user = user_builder::build_user(
        "alice".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012",
    );
    let group = group_builder::build_group(
        "admins".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012",
    );

    store.create_user(user).await.unwrap();
    store.create_group(group).await.unwrap();

    // Add user to group
    store.add_user_to_group("admins", "alice").await.unwrap();

    // List groups for user
    let groups = store.list_groups_for_user("alice").await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_name, "admins");

    // Remove user from group
    store
        .remove_user_from_group("admins", "alice")
        .await
        .unwrap();

    let groups_after = store.list_groups_for_user("alice").await.unwrap();
    assert_eq!(groups_after.len(), 0);
}

// ============================================================================
// ROLE STORE TESTS
// ============================================================================

#[tokio::test]
async fn test_role_create_and_get() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();
    let role = role_builder::build_role(
        "admin-role".to_string(),
        trust_policy,
        Some("/".to_string()),
        Some("Administrator role".to_string()),
        Some(3600),
        &provider,
        "123456789012",
    );

    let created = store.create_role(role.clone()).await.unwrap();
    assert_eq!(created.role_name, "admin-role");

    let retrieved = store.get_role("admin-role").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(
        retrieved.unwrap().description,
        Some("Administrator role".to_string())
    );
}

#[tokio::test]
async fn test_role_update() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();
    let role = role_builder::build_role(
        "test-role".to_string(),
        trust_policy,
        Some("/".to_string()),
        None,
        None,
        &provider,
        "123456789012",
    );

    store.create_role(role.clone()).await.unwrap();

    let updated = role_builder::update_description(role, Some("Updated description".to_string()));
    store.update_role(updated).await.unwrap();

    let retrieved = store.get_role("test-role").await.unwrap().unwrap();
    assert_eq!(
        retrieved.description,
        Some("Updated description".to_string())
    );
}

#[tokio::test]
async fn test_role_delete() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();
    let role = role_builder::build_role(
        "temp-role".to_string(),
        trust_policy,
        Some("/".to_string()),
        None,
        None,
        &provider,
        "123456789012",
    );

    store.create_role(role).await.unwrap();
    store.delete_role("temp-role").await.unwrap();

    let result = store.get_role("temp-role").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_role_list_multiple() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();

    for name in &["role-a", "role-b", "role-c"] {
        let role = role_builder::build_role(
            name.to_string(),
            trust_policy.clone(),
            Some("/".to_string()),
            None,
            None,
            &provider,
            "123456789012",
        );
        store.create_role(role).await.unwrap();
    }

    let (roles, is_truncated, _) = store.list_roles(None, None).await.unwrap();

    assert_eq!(roles.len(), 3);
    assert!(!is_truncated);
}

#[tokio::test]
async fn test_role_with_path_prefix() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();

    let role1 = role_builder::build_role(
        "service-role".to_string(),
        trust_policy.clone(),
        Some("/service/".to_string()),
        None,
        None,
        &provider,
        "123456789012",
    );
    let role2 = role_builder::build_role(
        "admin-role".to_string(),
        trust_policy,
        Some("/admin/".to_string()),
        None,
        None,
        &provider,
        "123456789012",
    );

    store.create_role(role1).await.unwrap();
    store.create_role(role2).await.unwrap();

    let (roles, _, _) = store.list_roles(Some("/service/"), None).await.unwrap();

    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].role_name, "service-role");
}

// ============================================================================
// SERVICE-LINKED ROLE STORE TESTS
// ============================================================================

#[tokio::test]
async fn test_service_linked_role_deletion_task() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();

    let task = slr_builder::build_deletion_task(
        "test-service-role".to_string(),
        &provider,
        "123456789012",
    );

    let task_id = task.deletion_task_id.clone();

    // Create deletion task
    store
        .create_service_linked_role_deletion_task(task.clone())
        .await
        .unwrap();

    // Get deletion task
    let retrieved = store
        .get_service_linked_role_deletion_task(&task_id)
        .await
        .unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().role_name, "test-service-role");
}

#[tokio::test]
async fn test_service_linked_role_deletion_task_nonexistent() {
    let store = InMemoryWamiStore::new();

    let result = store
        .get_service_linked_role_deletion_task("nonexistent-task")
        .await
        .unwrap();
    assert!(result.is_none());
}
