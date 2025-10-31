//! Hello WAMI - Absolute Basics
//!
//! This example demonstrates:
//! - Initializing an in-memory store
//! - Creating a single user
//! - Retrieving the user
//!
//! Scenario: Your first steps with WAMI - create and retrieve a user.
//!
//! Run with: `cargo run --example 01_hello_wami`

use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::UserStore;
use wami::wami::identity::user::builder::build_user;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hello WAMI ===\n");

    // Step 1: Initialize the store
    println!("Step 1: Initializing in-memory store...");
    let mut store = InMemoryWamiStore::default();
    println!("✓ Store initialized");

    // Step 2: Create a WamiContext for operations
    println!("\nStep 2: Creating WAMI context...");
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
    println!("✓ Context created");

    // Step 3: Build a user using pure functions
    println!("\nStep 3: Building user 'alice'...");
    let user = build_user("alice".to_string(), Some("/".to_string()), &context)?;
    println!("✓ User built with ARN: {}", user.wami_arn);

    // Step 4: Store the user
    println!("\nStep 4: Storing user in the store...");
    let created_user = store.create_user(user).await?;
    println!("✓ User stored successfully");
    println!("  - Name: {}", created_user.user_name);
    println!("  - User ID: {}", created_user.user_id);
    println!("  - ARN: {}", created_user.wami_arn);

    // Step 5: Retrieve the user
    println!("\nStep 5: Retrieving user from store...");
    let retrieved = store.get_user("alice").await?;
    match retrieved {
        Some(user) => {
            println!("✓ User retrieved successfully");
            println!("  - Name: {}", user.user_name);
            println!("  - Path: {}", user.path);
        }
        None => println!("✗ User not found"),
    }

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- InMemoryWamiStore provides a simple storage backend");
    println!("- Providers (AWS, GCP, Azure) handle platform-specific details");
    println!("- Pure functions create domain objects without side effects");

    Ok(())
}
