//! Role Assumption Workflow
//!
//! This example demonstrates:
//! - AssumeRole pattern for elevated permissions
//! - Temporary credentials with role permissions
//! - Session-based access
//!
//! Scenario: User assuming a role for elevated access.
//!
//! Run with: `cargo run --example 19_role_assumption_workflow`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{AssumeRoleService, RoleService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::role::requests::CreateRoleRequest;
use wami::wami::identity::user::requests::CreateUserRequest;
use wami::wami::sts::assume_role::requests::AssumeRoleRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Role Assumption Workflow ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create context
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

    let user_service = UserService::new(store.clone());
    let role_service = RoleService::new(store.clone());
    let sts_service = AssumeRoleService::new(store.clone());

    // Create user
    println!("Step 1: Creating user...\n");
    let alice = user_service
        .create_user(
            &context,
            CreateUserRequest {
                user_name: "alice".to_string(),
                path: Some("/".to_string()),
                permissions_boundary: None,
                tags: None,
            },
        )
        .await?;
    println!("✓ Created alice: {}", alice.arn);

    // Create elevated role
    println!("\nStep 2: Creating admin role...\n");
    let trust_policy = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"AWS":"*"},"Action":"sts:AssumeRole"}]}"#;
    let role = role_service
        .create_role(
            &context,
            CreateRoleRequest {
                role_name: "AdminRole".to_string(),
                path: Some("/".to_string()),
                assume_role_policy_document: trust_policy.to_string(),
                description: Some("Admin role for elevated access".to_string()),
                max_session_duration: Some(3600),
                permissions_boundary: None,
                tags: None,
            },
        )
        .await?;
    println!("✓ Created AdminRole: {}", role.arn);

    // Assume role
    println!("\nStep 3: Alice assuming AdminRole...\n");
    let assume_req = AssumeRoleRequest {
        role_arn: role.arn.clone(),
        role_session_name: "alice-admin-session".to_string(),
        duration_seconds: Some(3600),
        external_id: None,
        policy: None,
    };

    let response = sts_service
        .assume_role(&context, assume_req, &alice.arn)
        .await?;
    println!("✓ Successfully assumed role!");
    println!("  Assumed Role ARN: {}", response.assumed_role_user.arn);
    println!("  Access Key: {}", response.credentials.access_key_id);
    println!("  Expiration: {}", response.credentials.expiration);

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- AssumeRole provides temporary elevated permissions");
    println!("- Trust policies control who can assume roles");
    println!("- Session credentials expire automatically");

    Ok(())
}
