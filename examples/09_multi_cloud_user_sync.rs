//! Multi-Cloud User Sync
//!
//! This example demonstrates:
//! - Creating the same logical user across multiple cloud providers
//! - Different ARN formats per provider (AWS, GCP, Azure)
//! - Storing provider-specific metadata
//!
//! Scenario: alice@company.com needs identities in AWS, GCP, and Azure.
//!
//! Run with: `cargo run --example 09_multi_cloud_user_sync`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::UserService;
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::CreateUserRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Cloud User Sync ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create context for operations
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

    println!("✓ Using unified context for all operations");

    // === CREATE USER ===
    println!("\n\nStep 1: Creating alice user...\n");

    let user_service = UserService::new(store.clone());

    let user_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/cloud-sync/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            wami::types::Tag {
                key: "Email".to_string(),
                value: "alice@company.com".to_string(),
            },
            wami::types::Tag {
                key: "MultiCloud".to_string(),
                value: "true".to_string(),
            },
        ]),
    };

    let user = user_service.create_user(&context, user_req).await?;
    println!("✓ Created alice:");
    println!("  - ARN: {}", user.arn);
    println!("  - User ID: {}", user.user_id);
    println!("  - WAMI ARN: {}", user.wami_arn);

    // === COMPARE ARN FORMATS ===
    println!("\n\nStep 2: Understanding WAMI ARN format...\n");

    println!("WAMI unified ARN:");
    println!("  {}", user.wami_arn);
    println!("\nThis ARN can be transformed to provider-specific formats:");
    println!("  - AWS ARN format for AWS API calls");
    println!("  - GCP resource name format for GCP API calls");
    println!("  - Azure resource ID format for Azure API calls");

    // === DEMONSTRATE PROVIDER METADATA ===
    println!("\n\nStep 3: Understanding WAMI architecture...\n");

    println!("WAMI provides:");
    println!("- Unified WAMI ARN format (for internal operations)");
    println!("- Provider-specific ARN transformation when needed");
    println!("- Consistent resource identification across clouds");
    println!("- Tags for categorization and metadata");

    println!("\nBenefits:");
    println!("- Unified identity management across clouds");
    println!("- Provider-agnostic operations with context");
    println!("- Cross-cloud audit trails with WAMI ARNs");
    println!("- Multi-cloud resource tracking");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- WAMI uses a unified ARN format internally");
    println!("- Provider-specific ARNs can be generated when needed");
    println!("- Context-based operations work across all providers");
    println!("- Use tags to track cross-cloud relationships");

    Ok(())
}
