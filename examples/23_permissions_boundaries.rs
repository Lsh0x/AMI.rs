//! Example 23: Permissions Boundaries
//!
//! This example demonstrates how to use permissions boundaries to set the maximum
//! permissions that identity-based policies can grant to users and roles.
//!
//! Key Concepts:
//! - Permissions boundaries act as a ceiling for effective permissions
//! - Effective permissions = identity-based policies ∩ permissions boundary
//! - Both must allow an action for it to be permitted
//!
//! Run with: cargo run --example 23_permissions_boundaries

use std::sync::{Arc, RwLock};
use wami::{
    arn::{TenantPath, WamiArn},
    context::WamiContext,
    service::{
        EvaluationService, PermissionsBoundaryService, PolicyService, RoleService, UserService,
    },
    store::memory::InMemoryWamiStore,
    wami::identity::role::requests::CreateRoleRequest,
    wami::identity::user::requests::CreateUserRequest,
    wami::policies::evaluation::SimulatePrincipalPolicyRequest,
    wami::policies::permissions_boundary::{
        DeletePermissionsBoundaryRequest, PrincipalType, PutPermissionsBoundaryRequest,
    },
    wami::policies::policy::requests::CreatePolicyRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAMI Example 23: Permissions Boundaries ===\n");

    // Initialize store and services
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
    let boundary_service =
        PermissionsBoundaryService::new(store.clone(), "123456789012".to_string());
    let evaluation_service = EvaluationService::new(store.clone(), "123456789012".to_string());

    // ==========================================
    // Part 1: Create User with Admin Policy
    // ==========================================
    println!("📋 Part 1: Creating User with Admin Policy\n");

    // Create a user
    let alice_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/developers/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let alice = user_service.create_user(&context, alice_req).await?;
    println!("✅ Created user: {}", alice.user_name);
    println!("   ARN: {}\n", alice.arn);

    // Create an admin policy (allows all actions)
    let admin_policy_doc = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": "*",
            "Resource": "*"
        }]
    }"#;
    let admin_policy = policy_service
        .create_policy(
            &context,
            CreatePolicyRequest {
                policy_name: "AdminPolicy".to_string(),
                policy_document: admin_policy_doc.to_string(),
                path: Some("/".to_string()),
                description: Some("Full admin access".to_string()),
                tags: None,
            },
        )
        .await?;
    println!("✅ Created admin policy: {}", admin_policy.policy_name);
    println!("   ARN: {}", admin_policy.arn);
    println!("   Allows: All actions on all resources\n");

    // ==========================================
    // Part 2: Create S3-Only Boundary Policy
    // ==========================================
    println!("📋 Part 2: Creating S3-Only Boundary Policy\n");

    let s3_boundary_doc = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": "s3:*",
            "Resource": "*"
        }]
    }"#;
    let s3_boundary = policy_service
        .create_policy(
            &context,
            CreatePolicyRequest {
                policy_name: "S3OnlyBoundary".to_string(),
                policy_document: s3_boundary_doc.to_string(),
                path: Some("/boundaries/".to_string()),
                description: Some("Limits permissions to S3 only".to_string()),
                tags: None,
            },
        )
        .await?;
    println!("✅ Created boundary policy: {}", s3_boundary.policy_name);
    println!("   ARN: {}", s3_boundary.arn);
    println!("   Allows: Only S3 actions\n");

    // ==========================================
    // Part 3: Test Without Boundary
    // ==========================================
    println!("📋 Part 3: Testing Permissions WITHOUT Boundary\n");

    // Simulate alice's permissions (admin policy allows everything)
    let sim_req = SimulatePrincipalPolicyRequest {
        policy_source_arn: alice.arn.clone(),
        action_names: vec![
            "s3:GetObject".to_string(),
            "ec2:RunInstances".to_string(),
            "iam:CreateUser".to_string(),
        ],
        resource_arns: Some(vec!["*".to_string()]),
        policy_input_list: Some(vec![admin_policy_doc.to_string()]),
        context_entries: None,
    };

    let results = evaluation_service
        .simulate_principal_policy(sim_req)
        .await?;

    println!("Action Evaluation Results:");
    for result in &results.evaluation_results {
        println!(
            "  • {} on {} → {}",
            result.eval_action_name, result.eval_resource_name, result.eval_decision
        );
    }
    println!();

    // ==========================================
    // Part 4: Attach Boundary to User
    // ==========================================
    println!("📋 Part 4: Attaching S3-Only Boundary to User\n");

    let put_boundary_req = PutPermissionsBoundaryRequest {
        principal_type: PrincipalType::User,
        principal_name: "alice".to_string(),
        permissions_boundary: s3_boundary.arn.clone(),
    };
    boundary_service
        .put_permissions_boundary(put_boundary_req)
        .await?;
    println!("✅ Attached permissions boundary to alice");
    println!("   Boundary: {}", s3_boundary.arn);
    println!("   Effect: Alice's permissions are now limited to S3 actions only\n");

    // ==========================================
    // Part 5: Test WITH Boundary
    // ==========================================
    println!("📋 Part 5: Testing Permissions WITH Boundary\n");

    let sim_req_with_boundary = SimulatePrincipalPolicyRequest {
        policy_source_arn: alice.arn.clone(),
        action_names: vec![
            "s3:GetObject".to_string(),
            "s3:PutObject".to_string(),
            "ec2:RunInstances".to_string(),
            "iam:CreateUser".to_string(),
        ],
        resource_arns: Some(vec!["*".to_string()]),
        policy_input_list: Some(vec![admin_policy_doc.to_string()]),
        context_entries: None,
    };

    let results_with_boundary = evaluation_service
        .simulate_principal_policy(sim_req_with_boundary)
        .await?;

    println!("Action Evaluation Results (with boundary):");
    for result in &results_with_boundary.evaluation_results {
        let status = match result.eval_decision.as_str() {
            "allowed" => "✅ ALLOWED",
            "denied" => "❌ DENIED",
            _ => "⚠️  IMPLICIT DENY",
        };
        println!(
            "  {} → {} ({})",
            result.eval_action_name, status, result.eval_decision
        );
    }
    println!("\n📝 Notice:");
    println!("   • S3 actions are ALLOWED (both policy and boundary allow)");
    println!("   • EC2 and IAM actions are DENIED (boundary restricts them)\n");

    // ==========================================
    // Part 6: Boundary with Roles
    // ==========================================
    println!("📋 Part 6: Using Boundaries with Roles\n");

    // Create a role with assume policy
    let assume_policy_doc = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": {"Service": "ec2.amazonaws.com"},
            "Action": "sts:AssumeRole"
        }]
    }"#;

    let dev_role_req = CreateRoleRequest {
        role_name: "DeveloperRole".to_string(),
        assume_role_policy_document: assume_policy_doc.to_string(),
        path: Some("/roles/".to_string()),
        description: Some("Role for developers".to_string()),
        max_session_duration: None,
        permissions_boundary: None,
        tags: None,
    };
    let dev_role = role_service.create_role(&context, dev_role_req).await?;
    println!("✅ Created role: {}", dev_role.role_name);
    println!("   ARN: {}\n", dev_role.arn);

    // Create a read-only boundary
    let read_only_boundary_doc = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": [
                "s3:Get*",
                "s3:List*",
                "ec2:Describe*"
            ],
            "Resource": "*"
        }]
    }"#;
    let read_only_boundary = policy_service
        .create_policy(
            &context,
            CreatePolicyRequest {
                policy_name: "ReadOnlyBoundary".to_string(),
                policy_document: read_only_boundary_doc.to_string(),
                path: Some("/boundaries/".to_string()),
                description: Some("Limits to read-only operations".to_string()),
                tags: None,
            },
        )
        .await?;
    println!(
        "✅ Created read-only boundary: {}",
        read_only_boundary.policy_name
    );

    // Attach boundary to role
    let put_role_boundary = PutPermissionsBoundaryRequest {
        principal_type: PrincipalType::Role,
        principal_name: "DeveloperRole".to_string(),
        permissions_boundary: read_only_boundary.arn.clone(),
    };
    boundary_service
        .put_permissions_boundary(put_role_boundary)
        .await?;
    println!("✅ Attached read-only boundary to DeveloperRole\n");

    // Test role with boundary
    let role_sim_req = SimulatePrincipalPolicyRequest {
        policy_source_arn: dev_role.arn.clone(),
        action_names: vec![
            "s3:GetObject".to_string(),
            "s3:PutObject".to_string(),
            "ec2:DescribeInstances".to_string(),
            "ec2:RunInstances".to_string(),
        ],
        resource_arns: Some(vec!["*".to_string()]),
        policy_input_list: Some(vec![admin_policy_doc.to_string()]),
        context_entries: None,
    };

    let role_results = evaluation_service
        .simulate_principal_policy(role_sim_req)
        .await?;

    println!("Role Action Evaluation (with read-only boundary):");
    for result in &role_results.evaluation_results {
        let status = match result.eval_decision.as_str() {
            "allowed" => "✅ ALLOWED",
            "denied" => "❌ DENIED",
            _ => "⚠️  IMPLICIT DENY",
        };
        println!("  {} → {} ", result.eval_action_name, status);
    }
    println!("\n📝 Notice:");
    println!("   • Read operations (Get*, Describe*) are ALLOWED");
    println!("   • Write operations (Put*, Run*) are DENIED by boundary\n");

    // ==========================================
    // Part 7: Removing Boundaries
    // ==========================================
    println!("📋 Part 7: Removing Permissions Boundaries\n");

    let delete_user_boundary = DeletePermissionsBoundaryRequest {
        principal_type: PrincipalType::User,
        principal_name: "alice".to_string(),
    };
    boundary_service
        .delete_permissions_boundary(delete_user_boundary)
        .await?;
    println!("✅ Removed boundary from alice");
    println!("   Effect: Alice now has full admin permissions again\n");

    let delete_role_boundary = DeletePermissionsBoundaryRequest {
        principal_type: PrincipalType::Role,
        principal_name: "DeveloperRole".to_string(),
    };
    boundary_service
        .delete_permissions_boundary(delete_role_boundary)
        .await?;
    println!("✅ Removed boundary from DeveloperRole\n");

    // ==========================================
    // Part 8: Real-World Use Cases
    // ==========================================
    println!("📋 Part 8: Real-World Use Cases\n");
    println!("Use Case 1: Sandbox Environments");
    println!("  - Attach boundaries to prevent developers from:");
    println!("    • Creating IAM users/roles");
    println!("    • Modifying billing/account settings");
    println!("    • Accessing production resources\n");

    println!("Use Case 2: Contractor Access");
    println!("  - Limit contractors to specific services:");
    println!("    • Allow S3 and Lambda only");
    println!("    • Prevent infrastructure changes");
    println!("    • Ensure audit trail compliance\n");

    println!("Use Case 3: Delegated Administration");
    println!("  - Allow team leads to create users but:");
    println!("    • Boundary prevents privilege escalation");
    println!("    • New users inherit safe permission limits");
    println!("    • Central security team controls boundaries\n");

    println!("Use Case 4: Multi-Tenant SaaS");
    println!("  - Each tenant gets a boundary policy:");
    println!("    • Restricts access to tenant-specific resources");
    println!("    • Prevents cross-tenant data access");
    println!("    • Simplifies per-tenant permission management\n");

    println!("=== Example 23 Complete ===");
    println!("\n🎓 Key Takeaways:");
    println!("  1. Boundaries set maximum permissions (ceiling)");
    println!("  2. Effective permissions = identity policies ∩ boundary");
    println!("  3. Both identity policy AND boundary must allow an action");
    println!("  4. Boundaries prevent privilege escalation");
    println!("  5. Use for security controls, sandboxes, and delegation");

    Ok(())
}
