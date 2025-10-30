//! Basic CRUD Operations
//!
//! This example demonstrates:
//! - Creating multiple types of resources (users, groups, roles)
//! - Updating resources
//! - Deleting resources
//! - Listing resources
//!
//! Scenario: Managing a small team with users, groups, and roles.
//!
//! Run with: `cargo run --example 02_basic_crud_operations`

use wami::provider::AwsProvider;
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::{GroupStore, RoleStore, UserStore};
use wami::wami::identity::group::builder::build_group;
use wami::wami::identity::role::builder::build_role;
use wami::wami::identity::user::builder::build_user;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic CRUD Operations ===\n");

    let mut store = InMemoryWamiStore::default();
    let provider = AwsProvider::new();
    let account_id = "123456789012";

    // === CREATE Operations ===
    println!("Step 1: Creating resources...\n");

    // Create users
    println!("Creating users...");
    let alice = build_user(
        "alice".to_string(),
        Some("/users/".to_string()),
        &provider,
        account_id,
    );
    let bob = build_user(
        "bob".to_string(),
        Some("/users/".to_string()),
        &provider,
        account_id,
    );
    let charlie = build_user(
        "charlie".to_string(),
        Some("/users/".to_string()),
        &provider,
        account_id,
    );

    store.create_user(alice).await?;
    store.create_user(bob).await?;
    store.create_user(charlie).await?;
    println!("✓ Created 3 users: alice, bob, charlie");

    // Create groups
    println!("\nCreating groups...");
    let developers = build_group(
        "developers".to_string(),
        Some("/groups/".to_string()),
        &provider,
        account_id,
    );
    let admins = build_group(
        "admins".to_string(),
        Some("/groups/".to_string()),
        &provider,
        account_id,
    );

    store.create_group(developers).await?;
    store.create_group(admins).await?;
    println!("✓ Created 2 groups: developers, admins");

    // Create roles
    println!("\nCreating roles...");
    let deploy_role = build_role(
        "deploy-role".to_string(),
        r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"AWS":"*"},"Action":"sts:AssumeRole"}]}"#.to_string(),
        Some("/roles/".to_string()),
        None,
        None,
        &provider,
        account_id,
    );

    store.create_role(deploy_role).await?;
    println!("✓ Created 1 role: deploy-role");

    // === READ Operations ===
    println!("\n\nStep 2: Reading resources...\n");

    // List all users
    let (users, _, _) = store.list_users(None, None).await?;
    println!("✓ Found {} users:", users.len());
    for user in &users {
        println!("  - {}", user.user_name);
    }

    // Get specific user
    let alice_retrieved = store.get_user("alice").await?;
    if let Some(user) = alice_retrieved {
        println!("\n✓ Retrieved user 'alice':");
        println!("  - User ID: {}", user.user_id);
        println!("  - Path: {}", user.path);
        println!("  - ARN: {}", user.arn);
    }

    // List all groups
    let (groups, _, _) = store.list_groups(None, None).await?;
    println!("\n✓ Found {} groups:", groups.len());
    for group in &groups {
        println!("  - {}", group.group_name);
    }

    // === UPDATE Operations ===
    println!("\n\nStep 3: Updating resources...\n");

    // Update a user (change path)
    let mut alice = store.get_user("alice").await?.unwrap();
    let old_path = alice.path.clone();
    alice.path = "/admin-users/".to_string();
    store.update_user(alice).await?;
    println!(
        "✓ Updated alice's path from '{}' to '/admin-users/'",
        old_path
    );

    // Verify update
    let alice_updated = store.get_user("alice").await?.unwrap();
    println!("  Verified: path is now '{}'", alice_updated.path);

    // === DELETE Operations ===
    println!("\n\nStep 4: Deleting resources...\n");

    // Delete a user
    store.delete_user("charlie").await?;
    println!("✓ Deleted user 'charlie'");

    // Verify deletion
    let charlie_check = store.get_user("charlie").await?;
    if charlie_check.is_none() {
        println!("  Verified: charlie no longer exists");
    }

    // List users after deletion
    let (users_after, _, _) = store.list_users(None, None).await?;
    println!("\n✓ Remaining users: {}", users_after.len());
    for user in &users_after {
        println!("  - {}", user.user_name);
    }

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- CRUD operations are straightforward with store traits");
    println!("- List operations support pagination (not shown here)");
    println!("- Updates require fetching the resource first");
    println!("- Deletions are permanent (no soft delete by default)");

    Ok(())
}
