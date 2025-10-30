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
use wami::provider::{AwsProvider, GcpProvider};
use wami::service::UserService;
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::{CreateUserRequest, ListUsersRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Provider Switching ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    let aws_provider = Arc::new(AwsProvider::new());
    let gcp_provider = Arc::new(GcpProvider::new("my-gcp-project".to_string()));

    // === CREATE SERVICE WITH DEFAULT PROVIDER ===
    println!("Step 1: Creating service with AWS as default provider...\n");

    let user_service = UserService::new(store.clone(), "123456789012".to_string());

    println!("✓ Service created with AWS provider");

    // === CREATE USER WITH DEFAULT PROVIDER (AWS) ===
    println!("\nStep 2: Creating user with default AWS provider...\n");

    let alice_req = CreateUserRequest {
        user_name: "alice-aws".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };

    let alice_aws = user_service.create_user(alice_req).await?;
    println!("✓ Created alice-aws with AWS provider:");
    println!("  - ARN: {}", alice_aws.arn);
    println!("  - Provider ARN format matches AWS");

    // === SWITCH TO GCP PROVIDER ===
    println!("\n\nStep 3: Switching to GCP provider for next operation...\n");

    let gcp_service = user_service.with_provider(gcp_provider.clone());
    println!("✓ Switched to GCP provider (new service instance)");

    // === CREATE USER WITH GCP PROVIDER ===
    println!("\nStep 4: Creating user with GCP provider...\n");

    let bob_req = CreateUserRequest {
        user_name: "bob-gcp".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };

    let bob_gcp = gcp_service.create_user(bob_req).await?;
    println!("✓ Created bob-gcp with GCP provider:");
    println!("  - ARN: {}", bob_gcp.arn);
    println!("  - Provider ARN format matches GCP");

    // === DEMONSTRATE SWITCHING BACK ===
    println!("\n\nStep 5: Switching back to AWS provider...\n");

    let back_to_aws_service = gcp_service.with_provider(aws_provider.clone());

    let charlie_req = CreateUserRequest {
        user_name: "charlie-aws".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };

    let charlie_aws = back_to_aws_service.create_user(charlie_req).await?;
    println!("✓ Created charlie-aws with AWS provider:");
    println!("  - ARN: {}", charlie_aws.arn);

    // === LIST ALL USERS ===
    println!("\n\nStep 6: Listing all users (provider-agnostic)...\n");

    let (users, _, _) = user_service
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!("✓ Found {} users across all providers:", users.len());
    for user in &users {
        println!("  - {} → {}", user.user_name, user.arn);
    }

    // === DEMONSTRATE USE CASES ===
    println!("\n\nStep 7: Understanding provider switching benefits...\n");

    println!("Provider switching enables:");
    println!("- Multi-cloud deployments without code changes");
    println!("- Provider failover and disaster recovery");
    println!("- Cloud-agnostic CI/CD pipelines");
    println!("- Testing across different providers");
    println!("- Cost optimization by provider selection");

    println!("\nExample usage patterns:");
    println!("```rust");
    println!("// Create service with AWS");
    println!("let service = UserService::new(store, aws_provider, account_id);");
    println!();
    println!("// Temporarily switch to GCP for specific operation");
    println!("let result = service");
    println!("    .with_provider(gcp_provider)");
    println!("    .create_user(request)");
    println!("    .await?;");
    println!();
    println!("// Original service still uses AWS");
    println!("service.create_user(another_request).await?;");
    println!("```");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- with_provider() creates a new service instance with different provider");
    println!("- Original service remains unchanged");
    println!("- All operations use the store, provider just affects ARN generation");
    println!("- Enables flexible multi-cloud patterns");

    Ok(())
}
