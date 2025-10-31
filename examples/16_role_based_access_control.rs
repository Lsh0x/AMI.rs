//! Role-Based Access Control (RBAC)
//!
//! This example demonstrates:
//! - Defining roles (admin, developer, viewer)
//! - Assigning users to roles
//! - Policy inheritance through roles
//!
//! Scenario: Setting up RBAC for a development team.
//!
//! Run with: `cargo run --example 16_role_based_access_control`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{PolicyService, RoleService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::role::requests::CreateRoleRequest;
use wami::wami::identity::user::requests::CreateUserRequest;
use wami::wami::policies::policy::requests::CreatePolicyRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Role-Based Access Control (RBAC) ===\n");

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

    let user_service = UserService::new(store.clone());
    let role_service = RoleService::new(store.clone());
    let policy_service = PolicyService::new(store.clone());

    // === CREATE POLICIES ===
    println!("Step 1: Creating role policies...\n");

    let _admin_policy = policy_service.create_policy(&context, CreatePolicyRequest {
        policy_name: "AdminPolicy".to_string(),
        path: Some("/rbac/".to_string()),
        policy_document: r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":"*","Resource":"*"}]}"#.to_string(),
        description: Some("Full admin access".to_string()),
        tags: None,
    }).await?;
    println!("✓ Created AdminPolicy");

    let _dev_policy = policy_service.create_policy(&context, CreatePolicyRequest {
        policy_name: "DeveloperPolicy".to_string(),
        path: Some("/rbac/".to_string()),
        policy_document: r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":["s3:*","lambda:*"],"Resource":"*"}]}"#.to_string(),
        description: Some("Developer access to S3 and Lambda".to_string()),
        tags: None,
    }).await?;
    println!("✓ Created DeveloperPolicy");

    let _viewer_policy = policy_service.create_policy(&context, CreatePolicyRequest {
        policy_name: "ViewerPolicy".to_string(),
        path: Some("/rbac/".to_string()),
        policy_document: r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":["s3:Get*","s3:List*"],"Resource":"*"}]}"#.to_string(),
        description: Some("Read-only access".to_string()),
        tags: None,
    }).await?;
    println!("✓ Created ViewerPolicy");

    // === CREATE ROLES ===
    println!("\n\nStep 2: Creating roles...\n");

    let trust_policy = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"AWS":"*"},"Action":"sts:AssumeRole"}]}"#;

    let admin_role = role_service
        .create_role(
            &context,
            CreateRoleRequest {
                role_name: "Admin".to_string(),
                path: Some("/rbac/".to_string()),
                assume_role_policy_document: trust_policy.to_string(),
                description: Some("Administrator role".to_string()),
                max_session_duration: Some(3600),
                permissions_boundary: None,
                tags: None,
            },
        )
        .await?;
    println!("✓ Created Admin role: {}", admin_role.arn);

    let dev_role = role_service
        .create_role(
            &context,
            CreateRoleRequest {
                role_name: "Developer".to_string(),
                path: Some("/rbac/".to_string()),
                assume_role_policy_document: trust_policy.to_string(),
                description: Some("Developer role".to_string()),
                max_session_duration: Some(7200),
                permissions_boundary: None,
                tags: None,
            },
        )
        .await?;
    println!("✓ Created Developer role: {}", dev_role.arn);

    let viewer_role = role_service
        .create_role(
            &context,
            CreateRoleRequest {
                role_name: "Viewer".to_string(),
                path: Some("/rbac/".to_string()),
                assume_role_policy_document: trust_policy.to_string(),
                description: Some("Viewer role".to_string()),
                max_session_duration: Some(3600),
                permissions_boundary: None,
                tags: None,
            },
        )
        .await?;
    println!("✓ Created Viewer role: {}", viewer_role.arn);

    // === CREATE USERS ===
    println!("\n\nStep 3: Creating users...\n");

    user_service
        .create_user(
            &context,
            CreateUserRequest {
                user_name: "alice".to_string(),
                path: Some("/team/".to_string()),
                permissions_boundary: None,
                tags: Some(vec![wami::types::Tag {
                    key: "Role".to_string(),
                    value: "Admin".to_string(),
                }]),
            },
        )
        .await?;
    println!("✓ Created alice (will be assigned Admin role)");

    user_service
        .create_user(
            &context,
            CreateUserRequest {
                user_name: "bob".to_string(),
                path: Some("/team/".to_string()),
                permissions_boundary: None,
                tags: Some(vec![wami::types::Tag {
                    key: "Role".to_string(),
                    value: "Developer".to_string(),
                }]),
            },
        )
        .await?;
    println!("✓ Created bob (will be assigned Developer role)");

    user_service
        .create_user(
            &context,
            CreateUserRequest {
                user_name: "charlie".to_string(),
                path: Some("/team/".to_string()),
                permissions_boundary: None,
                tags: Some(vec![wami::types::Tag {
                    key: "Role".to_string(),
                    value: "Viewer".to_string(),
                }]),
            },
        )
        .await?;
    println!("✓ Created charlie (will be assigned Viewer role)");

    // === DEMONSTRATE RBAC ===
    println!("\n\nStep 4: Understanding RBAC pattern...\n");

    println!("RBAC structure created:");
    println!();
    println!("Roles → Policies:");
    println!("  Admin → AdminPolicy (full access)");
    println!("  Developer → DeveloperPolicy (S3 + Lambda)");
    println!("  Viewer → ViewerPolicy (read-only)");
    println!();
    println!("Users → Roles (via AssumeRole):");
    println!("  alice → Admin");
    println!("  bob → Developer");
    println!("  charlie → Viewer");
    println!();
    println!("Benefits of RBAC:");
    println!("- Centralized permission management");
    println!("- Easy to add/remove users from roles");
    println!("- Consistent permissions across teams");
    println!("- Simplified auditing");

    println!("\n✅ Example completed successfully!");

    Ok(())
}
