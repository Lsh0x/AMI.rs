//! Session Tokens
//!
//! This example demonstrates:
//! - Generating temporary session tokens
//! - Session expiration and lifecycle
//! - Refreshing credentials
//!
//! Scenario: Creating temporary credentials for a user.
//!
//! Run with: `cargo run --example 18_session_tokens`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{SessionTokenService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::CreateUserRequest;
use wami::wami::sts::session_token::requests::GetSessionTokenRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Session Tokens ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create context
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

    // Create user
    let user_service = UserService::new(store.clone());
    let alice_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let alice = user_service.create_user(&context, alice_req).await?;
    println!("Step 1: Created user alice");
    println!("  ARN: {}\n", alice.arn);

    // Generate session token
    let sts_service = SessionTokenService::new(store.clone());

    let token_req = GetSessionTokenRequest {
        duration_seconds: Some(3600),
        serial_number: None,
        token_code: None,
    };

    let response = sts_service
        .get_session_token(&context, token_req, &alice.arn)
        .await?;

    println!("Step 2: Generated session token");
    println!("  Access Key: {}", response.credentials.access_key_id);
    println!(
        "  Secret Key: {}...",
        &response.credentials.secret_access_key[..20]
    );
    println!(
        "  Session Token: {}...",
        &response.credentials.session_token[..30]
    );
    println!("  Expiration: {}", response.credentials.expiration);

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Session tokens provide temporary credentials");
    println!("- Credentials expire after specified duration");
    println!("- Useful for temporary or delegated access");

    Ok(())
}
