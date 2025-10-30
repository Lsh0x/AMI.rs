//! Tenant Migration
//!
//! This example demonstrates:
//! - Moving resources from one tenant to another
//! - Re-creating resources with new tenant context
//! - Updating resource references after migration
//!
//! Scenario: Migrating a user and their resources from old-tenant to new-tenant.
//!
//! Run with: `cargo run --example 08_tenant_migration`

use std::sync::{Arc, RwLock};
use wami::provider::AwsProvider;
use wami::service::{GroupService, TenantService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::group::requests::CreateGroupRequest;
use wami::wami::identity::user::requests::{CreateUserRequest, ListUsersRequest};
use wami::wami::tenant::model::TenantId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Tenant Migration ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    let _provider = Arc::new(AwsProvider::new());

    // === CREATE TENANTS ===
    println!("Step 1: Creating source and destination tenants...\n");

    let tenant_service = TenantService::new(store.clone(), "root".to_string());

    let _old_tenant_id = TenantId::new("old-tenant");
    tenant_service
        .create_tenant(
            "old-tenant".to_string(),
            Some("Old Tenant (deprecated)".to_string()),
            None,
        )
        .await?;
    println!("✓ Created source tenant: old-tenant");

    let _new_tenant_id = TenantId::new("new-tenant");
    tenant_service
        .create_tenant(
            "new-tenant".to_string(),
            Some("New Tenant (target)".to_string()),
            None,
        )
        .await?;
    println!("✓ Created destination tenant: new-tenant");

    // === CREATE RESOURCES IN OLD TENANT ===
    println!("\nStep 2: Creating resources in old tenant...\n");

    let old_user_service = UserService::new(store.clone(), "old-tenant".to_string());
    let old_group_service = GroupService::new(store.clone(), "old-tenant".to_string());

    // Create user
    let user_req = CreateUserRequest {
        user_name: "bob".to_string(),
        path: Some("/users/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let old_user = old_user_service.create_user(user_req).await?;
    println!("✓ Created user in old-tenant:");
    println!("  - Name: {}", old_user.user_name);
    println!("  - ARN: {}", old_user.arn);

    // Create group
    let group_req = CreateGroupRequest {
        group_name: "developers".to_string(),
        path: Some("/groups/".to_string()),
        tags: None,
    };
    let old_group = old_group_service.create_group(group_req).await?;
    println!("\n✓ Created group in old-tenant:");
    println!("  - Name: {}", old_group.group_name);
    println!("  - ARN: {}", old_group.arn);

    // Add user to group
    old_group_service
        .add_user_to_group("developers", "bob")
        .await?;
    println!("\n✓ Added bob to developers group in old-tenant");

    // === MIGRATE TO NEW TENANT ===
    println!("\n\nStep 3: Migrating resources to new tenant...\n");

    let new_user_service = UserService::new(store.clone(), "new-tenant".to_string());
    let new_group_service = GroupService::new(store.clone(), "new-tenant".to_string());

    // Re-create user in new tenant
    println!("Migrating user...");
    let new_user_req = CreateUserRequest {
        user_name: old_user.user_name.clone(),
        path: Some(old_user.path.clone()),
        permissions_boundary: old_user.permissions_boundary.clone(),
        tags: Some(old_user.tags.clone()),
    };
    let new_user = new_user_service.create_user(new_user_req).await?;
    println!("✓ Re-created user in new-tenant:");
    println!("  - Old ARN: {}", old_user.arn);
    println!("  - New ARN: {}", new_user.arn);

    // Re-create group in new tenant
    println!("\nMigrating group...");
    let new_group_req = CreateGroupRequest {
        group_name: old_group.group_name.clone(),
        path: Some(old_group.path.clone()),
        tags: Some(old_group.tags.clone()),
    };
    let new_group = new_group_service.create_group(new_group_req).await?;
    println!("✓ Re-created group in new-tenant:");
    println!("  - Old ARN: {}", old_group.arn);
    println!("  - New ARN: {}", new_group.arn);

    // Re-establish group membership
    println!("\nRestoring group membership...");
    new_group_service
        .add_user_to_group("developers", "bob")
        .await?;
    println!("✓ Re-added bob to developers group in new-tenant");

    // === CLEANUP OLD TENANT (Optional) ===
    println!("\n\nStep 4: Cleaning up old tenant (optional)...\n");

    println!("In production, you would:");
    println!("- Remove user from old group");
    println!("- Delete user from old tenant");
    println!("- Delete group from old tenant");
    println!("- Audit all resource references");
    println!("- Update application configurations");

    // Example cleanup (commented to preserve state for demonstration)
    // old_group_service.remove_user_from_group("developers", "bob").await?;
    // old_user_service.delete_user("bob").await?;
    // old_group_service.delete_group("developers").await?;
    println!("\n(Cleanup skipped for demonstration purposes)");

    // === VERIFICATION ===
    println!("\n\nStep 5: Verifying migration...\n");

    let (old_users, _, _) = old_user_service
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!("Users remaining in old-tenant: {}", old_users.len());

    let (new_users, _, _) = new_user_service
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!("Users now in new-tenant: {}", new_users.len());
    for user in &new_users {
        println!("  - {}", user.user_name);
    }

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Tenant migration requires re-creating resources in the target tenant");
    println!("- ARNs change when resources move between tenants");
    println!("- Preserve metadata (tags, paths) during migration");
    println!("- Update all references after migration");
    println!("- Consider phased migration for large-scale moves");

    Ok(())
}
