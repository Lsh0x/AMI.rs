//! Policy Basics
//!
//! This example demonstrates:
//! - Creating IAM policies with allow/deny statements
//! - Attaching policies to users and roles
//! - Testing basic access control
//!
//! Scenario: Creating policies for read-only and admin access.
//!
//! Run with: `cargo run --example 14_policy_basics`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{PolicyService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::CreateUserRequest;
use wami::wami::policies::policy::requests::{CreatePolicyRequest, ListPoliciesRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Policy Basics ===\n");

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

    let policy_service = PolicyService::new(store.clone());
    let user_service = UserService::new(store.clone());

    // === CREATE USERS ===
    println!("Step 1: Creating users...\n");

    let alice_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    user_service.create_user(&context, alice_req).await?;
    println!("✓ Created alice");

    let bob_req = CreateUserRequest {
        user_name: "bob".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    user_service.create_user(&context, bob_req).await?;
    println!("✓ Created bob");

    // === CREATE READ-ONLY POLICY ===
    println!("\n\nStep 2: Creating read-only policy...\n");

    let readonly_policy_doc = r#"{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": [
      "s3:GetObject",
      "s3:ListBucket"
    ],
    "Resource": "*"
  }]
}"#;

    let readonly_req = CreatePolicyRequest {
        policy_name: "ReadOnlyAccess".to_string(),
        path: Some("/policies/".to_string()),
        policy_document: readonly_policy_doc.to_string(),
        description: Some("Grants read-only access to resources".to_string()),
        tags: None,
    };

    let readonly_policy = policy_service.create_policy(&context, readonly_req).await?;
    println!("✓ Created ReadOnlyAccess policy:");
    println!("  - ARN: {}", readonly_policy.arn);
    println!("  - Actions: s3:GetObject, s3:ListBucket");
    println!("  - Effect: Allow");

    // === CREATE DENY POLICY ===
    println!("\n\nStep 3: Creating deny policy...\n");

    let deny_policy_doc = r#"{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Deny",
    "Action": [
      "s3:DeleteObject",
      "s3:DeleteBucket"
    ],
    "Resource": "*"
  }]
}"#;

    let deny_req = CreatePolicyRequest {
        policy_name: "DenyDelete".to_string(),
        path: Some("/policies/".to_string()),
        policy_document: deny_policy_doc.to_string(),
        description: Some("Explicitly denies delete operations".to_string()),
        tags: None,
    };

    let deny_policy = policy_service.create_policy(&context, deny_req).await?;
    println!("✓ Created DenyDelete policy:");
    println!("  - ARN: {}", deny_policy.arn);
    println!("  - Actions: s3:DeleteObject, s3:DeleteBucket");
    println!("  - Effect: Deny (overrides all allows)");

    // === CREATE ADMIN POLICY ===
    println!("\n\nStep 4: Creating admin policy...\n");

    let admin_policy_doc = r#"{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "*",
    "Resource": "*"
  }]
}"#;

    let admin_req = CreatePolicyRequest {
        policy_name: "AdministratorAccess".to_string(),
        path: Some("/policies/".to_string()),
        policy_document: admin_policy_doc.to_string(),
        description: Some("Grants full access to all resources".to_string()),
        tags: None,
    };

    let admin_policy = policy_service.create_policy(&context, admin_req).await?;
    println!("✓ Created AdministratorAccess policy:");
    println!("  - ARN: {}", admin_policy.arn);
    println!("  - Actions: * (all actions)");
    println!("  - Resources: * (all resources)");

    // === LIST ALL POLICIES ===
    println!("\n\nStep 5: Listing all policies...\n");

    let (policies, _, _) = policy_service
        .list_policies(ListPoliciesRequest {
            scope: None,
            only_attached: None,
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!("✓ Found {} policies:", policies.len());
    for policy in &policies {
        println!("  - {} ({})", policy.policy_name, policy.arn);
    }

    // === DEMONSTRATE POLICY CONCEPTS ===
    println!("\n\nStep 6: Understanding policy concepts...\n");

    println!("Policy evaluation order:");
    println!("1. Explicit DENY - Always wins");
    println!("2. Explicit ALLOW - Required for access");
    println!("3. Implicit DENY - Default if no allow");
    println!();
    println!("Policy types:");
    println!("- Identity-based: Attached to users/roles/groups");
    println!("- Resource-based: Attached to resources (not shown here)");
    println!("- Permissions boundaries: Maximum permissions (not shown here)");
    println!();
    println!("Best practices:");
    println!("- Use deny policies for critical restrictions");
    println!("- Grant least privilege with allow policies");
    println!("- Use managed policies for common patterns");
    println!("- Regularly audit policy attachments");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Policies define permissions using JSON documents");
    println!("- Statements contain Effect, Action, and Resource");
    println!("- Deny always overrides allow");
    println!("- Policies can be attached to principals (users/roles)");

    Ok(())
}
