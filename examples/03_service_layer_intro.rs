//! Service Layer Introduction
//!
//! This example demonstrates:
//! - Using the service layer instead of direct store access
//! - Benefits of the service abstraction
//! - Thread-safe concurrent access with Arc<RwLock<Store>>
//!
//! Scenario: Same operations as example 02, but using the service layer.
//!
//! Run with: `cargo run --example 03_service_layer_intro`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{GroupService, RoleService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::group::requests::CreateGroupRequest;
use wami::wami::identity::role::requests::CreateRoleRequest;
use wami::wami::identity::user::requests::{CreateUserRequest, ListUsersRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Service Layer Introduction ===\n");

    // Step 1: Initialize store with Arc<RwLock> for thread-safe access
    println!("Step 1: Initializing services...\n");
    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create a WamiContext for operations
    let context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single("root"))
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single("root"))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Create services
    let user_service = UserService::new(store.clone());
    let group_service = GroupService::new(store.clone());
    let role_service = RoleService::new(store.clone());

    println!("✓ Services initialized");

    // === CREATE Operations via Services ===
    println!("\nStep 2: Creating resources via services...\n");

    // Create users
    println!("Creating users...");
    let alice_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/users/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let alice = user_service.create_user(&context, alice_req).await?;
    println!("✓ Created user: {}", alice.user_name);

    let bob_req = CreateUserRequest {
        user_name: "bob".to_string(),
        path: Some("/users/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    user_service.create_user(&context, bob_req).await?;
    println!("✓ Created user: bob");

    // Create groups
    println!("\nCreating groups...");
    let dev_group_req = CreateGroupRequest {
        group_name: "developers".to_string(),
        path: Some("/groups/".to_string()),
        tags: None,
    };
    let dev_group = group_service.create_group(&context, dev_group_req).await?;
    println!("✓ Created group: {}", dev_group.group_name);

    // Create role
    println!("\nCreating role...");
    let role_req = CreateRoleRequest {
        role_name: "deploy-role".to_string(),
        path: Some("/roles/".to_string()),
        assume_role_policy_document: r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"AWS":"*"},"Action":"sts:AssumeRole"}]}"#.to_string(),
        description: Some("Role for deployment".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: None,
    };
    let role = role_service.create_role(&context, role_req).await?;
    println!("✓ Created role: {}", role.role_name);

    // === READ Operations via Services ===
    println!("\n\nStep 3: Reading resources via services...\n");

    // Get specific user
    let alice_retrieved = user_service.get_user("alice").await?;
    if let Some(user) = alice_retrieved {
        println!("✓ Retrieved user 'alice':");
        println!("  - User ID: {}", user.user_id);
        println!("  - ARN: {}", user.arn);
    }

    // List users
    let users = user_service
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!("\n✓ Found {} users via service:", users.0.len());
    for user in &users.0 {
        println!("  - {}", user.user_name);
    }

    // === UPDATE Operations via Services ===
    println!("\n\nStep 4: Updating resources via services...\n");

    use wami::wami::identity::user::requests::UpdateUserRequest;
    let update_req = UpdateUserRequest {
        user_name: "alice".to_string(),
        new_user_name: None,
        new_path: Some("/admin-users/".to_string()),
    };
    user_service.update_user(update_req).await?;
    println!("✓ Updated alice's path to '/admin-users/'");

    // === DELETE Operations via Services ===
    println!("\n\nStep 5: Deleting resources via services...\n");

    user_service.delete_user("bob").await?;
    println!("✓ Deleted user 'bob'");

    // Verify deletion
    let bob_check = user_service.get_user("bob").await?;
    if bob_check.is_none() {
        println!("  Verified: bob no longer exists");
    }

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Services provide a higher-level API than raw store access");
    println!("- Arc<RwLock<Store>> enables thread-safe concurrent operations");
    println!("- Services use request/response DTOs for clean API contracts");
    println!("- Services encapsulate business logic and validation");

    Ok(())
}
