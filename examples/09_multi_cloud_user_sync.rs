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
use wami::provider::{AwsProvider, AzureProvider, GcpProvider};
use wami::service::UserService;
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::CreateUserRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Cloud User Sync ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // === CREATE PROVIDERS ===
    println!("Step 1: Initializing cloud providers...\n");

    let _aws_provider = Arc::new(AwsProvider::new());
    let _gcp_provider = Arc::new(GcpProvider::new("my-project".to_string()));
    let _azure_provider = Arc::new(AzureProvider::new(
        "my-subscription".to_string(),
        "default-rg".to_string(),
    ));

    println!("✓ Initialized 3 providers:");
    println!("  - AWS");
    println!("  - GCP (project: my-project)");
    println!("  - Azure (subscription: my-subscription)");

    // === CREATE USER IN AWS ===
    println!("\n\nStep 2: Creating alice in AWS...\n");

    let aws_service = UserService::new(
        store.clone(),
        "123456789012".to_string(), // AWS account ID
    );

    let aws_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/cloud-sync/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            wami::types::Tag {
                key: "Provider".to_string(),
                value: "AWS".to_string(),
            },
            wami::types::Tag {
                key: "Email".to_string(),
                value: "alice@company.com".to_string(),
            },
        ]),
    };

    let aws_user = aws_service.create_user(aws_req).await?;
    println!("✓ Created alice in AWS:");
    println!("  - ARN: {}", aws_user.arn);
    println!("  - User ID: {}", aws_user.user_id);
    println!("  - WAMI ARN: {}", aws_user.wami_arn);

    // === CREATE USER IN GCP ===
    println!("\n\nStep 3: Creating alice in GCP...\n");

    let gcp_service = UserService::new(
        store.clone(),
        "my-project".to_string(), // GCP project ID
    );

    let gcp_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/cloud-sync/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            wami::types::Tag {
                key: "Provider".to_string(),
                value: "GCP".to_string(),
            },
            wami::types::Tag {
                key: "Email".to_string(),
                value: "alice@company.com".to_string(),
            },
        ]),
    };

    let gcp_user = gcp_service.create_user(gcp_req).await?;
    println!("✓ Created alice in GCP:");
    println!("  - ARN: {}", gcp_user.arn);
    println!("  - User ID: {}", gcp_user.user_id);
    println!("  - WAMI ARN: {}", gcp_user.wami_arn);

    // === CREATE USER IN AZURE ===
    println!("\n\nStep 4: Creating alice in Azure...\n");

    let azure_service = UserService::new(
        store.clone(),
        "my-subscription".to_string(), // Azure subscription ID
    );

    let azure_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/cloud-sync/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            wami::types::Tag {
                key: "Provider".to_string(),
                value: "Azure".to_string(),
            },
            wami::types::Tag {
                key: "Email".to_string(),
                value: "alice@company.com".to_string(),
            },
        ]),
    };

    let azure_user = azure_service.create_user(azure_req).await?;
    println!("✓ Created alice in Azure:");
    println!("  - ARN: {}", azure_user.arn);
    println!("  - User ID: {}", azure_user.user_id);
    println!("  - WAMI ARN: {}", azure_user.wami_arn);

    // === COMPARE ARN FORMATS ===
    println!("\n\nStep 5: Comparing ARN formats across providers...\n");

    println!("AWS ARN format:");
    println!("  {}", aws_user.arn);
    println!("\nGCP ARN format:");
    println!("  {}", gcp_user.arn);
    println!("\nAzure ARN format:");
    println!("  {}", azure_user.arn);
    println!("\nWAMI unified ARN (all use same format):");
    println!("  AWS:   {}", aws_user.wami_arn);
    println!("  GCP:   {}", gcp_user.wami_arn);
    println!("  Azure: {}", azure_user.wami_arn);

    // === DEMONSTRATE PROVIDER METADATA ===
    println!("\n\nStep 6: Understanding provider metadata...\n");

    println!("Each user stores provider-specific information:");
    println!("- Native provider ARN (for cloud API calls)");
    println!("- WAMI ARN (for unified identity)");
    println!("- Provider-specific resource IDs");
    println!("- Tags for categorization and metadata");

    println!("\nProvider metadata enables:");
    println!("- Unified identity across clouds");
    println!("- Native API integration per provider");
    println!("- Cross-cloud audit trails");
    println!("- Multi-cloud cost allocation");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Same logical user can exist in multiple clouds");
    println!("- Each provider has its own ARN format");
    println!("- WAMI ARNs provide a unified identifier");
    println!("- Use tags to track cross-cloud relationships");

    Ok(())
}
