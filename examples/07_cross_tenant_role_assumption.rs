//! Cross-Tenant Role Assumption
//!
//! This example demonstrates:
//! - User in one tenant assuming a role in another tenant
//! - STS temporary credentials with cross-tenant access
//! - Trust policies enabling cross-tenant access
//!
//! Scenario: A user from Company A needs temporary access to Company B resources.
//!
//! Run with: `cargo run --example 07_cross_tenant_role_assumption`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{AssumeRoleService, RoleService, TenantService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::role::requests::CreateRoleRequest;
use wami::wami::identity::user::requests::CreateUserRequest;
use wami::wami::sts::assume_role::requests::AssumeRoleRequest;
use wami::wami::tenant::model::TenantId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cross-Tenant Role Assumption ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create root context
    let root_context = WamiContext::builder()
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
        .is_root(true)
        .build()?;

    // Create tenant A context
    let tenant_a_context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single(10000000))
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(10000000))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Create tenant B context
    let tenant_b_context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single(20000000))
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(20000000))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // === SETUP TENANTS ===
    println!("Step 1: Creating two tenants...\n");

    let tenant_service = TenantService::new(store.clone());

    // Tenant A (source)
    // Tenant IDs are now numeric - use actual IDs from created tenants
    // For this example, we'll use placeholder IDs that match the context
    let _tenant_a_id = TenantId::from_string("10000000").unwrap();
    tenant_service
        .create_tenant(
            &root_context,
            "company-a".to_string(),
            Some("Company A".to_string()),
            None,
        )
        .await?;
    println!("✓ Created tenant: company-a");

    // Tenant B (target)
    let _tenant_b_id = TenantId::from_string("20000000").unwrap();
    tenant_service
        .create_tenant(
            &root_context,
            "company-b".to_string(),
            Some("Company B".to_string()),
            None,
        )
        .await?;
    println!("✓ Created tenant: company-b");

    // === CREATE USER IN TENANT A ===
    println!("\nStep 2: Creating user in tenant A...\n");

    let user_service = UserService::new(store.clone());

    let alice_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let alice = user_service
        .create_user(&tenant_a_context, alice_req)
        .await?;
    println!("✓ Created alice in company-a");
    println!("  ARN: {}", alice.arn);

    // === CREATE ROLE IN TENANT B WITH TRUST POLICY ===
    println!("\nStep 3: Creating cross-tenant role in tenant B...\n");

    let role_service = RoleService::new(store.clone());

    // Trust policy allowing Company A users to assume this role
    let trust_policy = r#"{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": {"AWS": "arn:aws:iam::company-a:root"},
    "Action": "sts:AssumeRole"
  }]
}"#
    .to_string();

    let role_req = CreateRoleRequest {
        role_name: "cross-tenant-reader".to_string(),
        path: Some("/".to_string()),
        assume_role_policy_document: trust_policy,
        description: Some("Role for cross-tenant read access".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: None,
    };

    let role = role_service
        .create_role(&tenant_b_context, role_req)
        .await?;
    println!("✓ Created cross-tenant-reader role in company-b");
    println!("  Role ARN: {}", role.arn);

    // === ASSUME ROLE ACROSS TENANTS ===
    println!("\nStep 4: Alice assuming role in company-b...\n");

    let sts_service = AssumeRoleService::new(store.clone());

    let assume_req = AssumeRoleRequest {
        role_arn: role.arn.clone(),
        role_session_name: "alice-cross-tenant-session".to_string(),
        duration_seconds: Some(3600),
        external_id: None,
        policy: None,
    };

    let assume_response = sts_service
        .assume_role(&tenant_a_context, assume_req, &alice.arn)
        .await?;
    println!("✓ Alice successfully assumed cross-tenant role!");
    println!("  Credentials:");
    println!(
        "    - Access Key: {}",
        assume_response.credentials.access_key_id
    );
    println!(
        "    - Session Token: {}...",
        &assume_response.credentials.session_token[..20]
    );
    println!(
        "    - Expiration: {}",
        assume_response.credentials.expiration
    );
    println!("\n  Assumed Role:");
    println!("    - ARN: {}", assume_response.assumed_role_user.arn);
    println!(
        "    - User ID: {}",
        assume_response.assumed_role_user.assumed_role_id
    );

    // === DEMONSTRATE USE CASE ===
    println!("\n\nStep 5: Understanding cross-tenant scenarios...\n");

    println!("Cross-tenant role assumption enables:");
    println!("- Partner company collaboration");
    println!("- Outsourced operations (DevOps team accessing client resources)");
    println!("- Centralized security auditing across organizations");
    println!("- Merger & acquisition transitional access");
    println!("\nSecurity considerations:");
    println!("- Trust policies must explicitly allow cross-tenant access");
    println!("- Use external_id for additional security");
    println!("- Limit session duration to minimum required");
    println!("- Attach restrictive permission policies to the role");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Cross-tenant access requires explicit trust policies");
    println!("- STS provides temporary, limited-privilege credentials");
    println!("- AssumeRole creates a session with the target role's permissions");
    println!("- Credentials are time-limited and automatically expire");

    Ok(())
}
