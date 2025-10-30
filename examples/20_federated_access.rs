//! Federated Access
//!
//! This example demonstrates:
//! - External identity federation
//! - Federation token generation
//! - Temporary credentials for external users
//!
//! Scenario: Granting temporary access to external partner.
//!
//! Run with: `cargo run --example 20_federated_access`

use std::sync::{Arc, RwLock};
use wami::provider::AwsProvider;
use wami::service::{FederationService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::CreateUserRequest;
use wami::wami::sts::federation::requests::GetFederationTokenRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Federated Access ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    let _provider = Arc::new(AwsProvider::new());
    let account_id = "123456789012";

    // Create admin user who will generate federation tokens
    let user_service = UserService::new(store.clone(), account_id.to_string());
    let admin = user_service
        .create_user(CreateUserRequest {
            user_name: "admin".to_string(),
            path: Some("/".to_string()),
            permissions_boundary: None,
            tags: None,
        })
        .await?;

    let fed_service = FederationService::new(store, account_id.to_string());

    println!("Step 1: Generating federation token for external user...\n");

    let fed_req = GetFederationTokenRequest {
        name: "partner-user".to_string(),
        duration_seconds: Some(3600),
        policy: Some(r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":"s3:GetObject","Resource":"*"}]}"#.to_string()),
    };

    let response = fed_service
        .get_federation_token(fed_req, &admin.arn)
        .await?;

    println!("✓ Generated federation token:");
    println!("  Federated User ARN: {}", response.federated_user.arn);
    println!("  Access Key: {}", response.credentials.access_key_id);
    println!("  Expiration: {}", response.credentials.expiration);

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Federation enables external user access");
    println!("- Temporary credentials with limited permissions");
    println!("- Useful for partner integrations");

    Ok(())
}
