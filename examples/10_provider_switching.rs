//! Provider Switching
//!
//! This example demonstrates:
//! - Using service.with_provider() to dynamically switch providers
//! - Same service, different cloud backends
//! - Provider-specific feature handling
//!
//! Scenario: Operations team managing resources across AWS and GCP.
//!
//! Run with: `cargo run --example 10_provider_switching`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::UserService;
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::{CreateUserRequest, ListUsersRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Provider Switching (Multi-Context) ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create a WamiContext for operations
    let context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single(0))
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(0))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // === CREATE SERVICE ===
    println!("Step 1: Creating service...\n");

    let user_service = UserService::new(store.clone());

    println!("✓ Service created");

    // === CREATE USER WITH CONTEXT ===
    println!("\nStep 2: Creating user with context...\n");

    let alice_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };

    let alice = user_service.create_user(&context, alice_req).await?;
    println!("✓ Created alice:");
    println!("  - ARN: {}", alice.arn);
    println!("  - WAMI ARN: {}", alice.wami_arn);

    // === CREATE ANOTHER USER ===
    println!("\nStep 3: Creating another user...\n");

    let bob_req = CreateUserRequest {
        user_name: "bob".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };

    let bob = user_service.create_user(&context, bob_req).await?;
    println!("✓ Created bob:");
    println!("  - ARN: {}", bob.arn);
    println!("  - WAMI ARN: {}", bob.wami_arn);

    // === LIST ALL USERS ===
    println!("\n\nStep 4: Listing all users...\n");

    let (users, _, _) = user_service
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!("✓ Found {} users:", users.len());
    for user in &users {
        println!("  - {} → {}", user.user_name, user.arn);
    }

    // === DEMONSTRATE USE CASES ===
    println!("\n\nStep 5: Understanding WAMI architecture...\n");

    println!("WAMI enables:");
    println!("- Multi-cloud deployments with unified ARN format");
    println!("- Provider-agnostic operations through WamiContext");
    println!("- Cloud-agnostic CI/CD pipelines");
    println!("- Consistent resource identification across providers");
    println!("- Flexible multi-tenant and multi-cloud patterns");

    println!("\nExample usage patterns:");
    println!("```rust");
    println!("// Create context with instance and tenant information");
    println!("let context = WamiContext::builder()");
    println!("    .instance_id(\"123456789012\")");
    println!("    .tenant_path(TenantPath::single(\"root\"))");
    println!("    .caller_arn(...)");
    println!("    .build()?;");
    println!();
    println!("// Use context for all operations");
    println!("let user = service.create_user(&context, request).await?;");
    println!("```");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Services use WamiContext instead of providers");
    println!("- Context contains instance, tenant, and caller information");
    println!("- All operations use WAMI ARNs for consistent identification");
    println!("- Provider-specific ARNs can be generated when needed");

    Ok(())
}
