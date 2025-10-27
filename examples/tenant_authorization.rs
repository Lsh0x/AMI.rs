//! Tenant Authorization Examples
//!
//! This example demonstrates how to use the IAM-based tenant authorization system
//! both with and without a store, showcasing the flexibility of the unified approach.

use std::sync::Arc;
use wami::provider::AwsProvider;
use wami::tenant::authorization::{
    build_tenant_admin_policy, build_tenant_readonly_policy, TenantAction, TenantAuthorizer,
};
use wami::{CreatePolicyRequest, IamClient, InMemoryStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Tenant Authorization Examples ===\n");

    // Example 1: Standalone authorization (without store)
    standalone_authorization().await?;

    // Example 2: Authorization with IAM store
    authorization_with_store().await?;

    // Example 3: Complex multi-policy scenario
    complex_authorization_scenario().await?;

    Ok(())
}

/// Example 1: Standalone authorization without a store
///
/// This shows how you can use the authorization system with inline policies,
/// without needing any persistent storage.
async fn standalone_authorization() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 1: Standalone Authorization (No Store)\n");

    // Create authorization policies inline
    let engineering_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": ["tenant:Read", "tenant:Update", "tenant:ManageUsers"],
            "Resource": "arn:wami:tenant::acme/engineering/*"
        }]
    }"#;

    let production_readonly_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": "tenant:Read",
            "Resource": "arn:wami:tenant::acme/production/*"
        }]
    }"#;

    // Create authorizer with these policies
    let authorizer = TenantAuthorizer::new(vec![
        engineering_policy.to_string(),
        production_readonly_policy.to_string(),
    ]);

    // Alice is an engineering admin
    let alice_arn = "arn:aws:iam::123456789012:user/alice";

    // Check various permissions
    let can_read_eng = authorizer
        .check_permission(alice_arn, "acme/engineering/frontend", TenantAction::Read)
        .await?;
    println!("✓ Alice can read engineering tenant: {}", can_read_eng);

    let can_manage_eng = authorizer
        .check_permission(
            alice_arn,
            "acme/engineering/frontend",
            TenantAction::ManageUsers,
        )
        .await?;
    println!("✓ Alice can manage engineering users: {}", can_manage_eng);

    let can_delete_eng = authorizer
        .check_permission(alice_arn, "acme/engineering/frontend", TenantAction::Delete)
        .await?;
    println!(
        "✗ Alice CANNOT delete engineering tenant: {}",
        !can_delete_eng
    );

    let can_read_prod = authorizer
        .check_permission(alice_arn, "acme/production/api", TenantAction::Read)
        .await?;
    println!("✓ Alice can read production tenant: {}", can_read_prod);

    let can_update_prod = authorizer
        .check_permission(alice_arn, "acme/production/api", TenantAction::Update)
        .await?;
    println!(
        "✗ Alice CANNOT update production tenant: {}",
        !can_update_prod
    );

    println!();
    Ok(())
}

/// Example 2: Authorization using IAM policies stored in a store
///
/// This shows how you can use the IAM system to persist and manage
/// authorization policies.
async fn authorization_with_store() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 2: Authorization with IAM Store\n");

    // Create store and IAM client
    let store = InMemoryStore::new();
    let provider = Arc::new(AwsProvider::default());
    let mut iam_client = IamClient::with_provider(store, provider);

    // Create a tenant admin policy in IAM
    let admin_policy_request = CreatePolicyRequest {
        policy_name: "TenantAdminPolicy".to_string(),
        policy_document: build_tenant_admin_policy("acme"),
        path: Some("/tenant/".to_string()),
        description: Some("Full admin access to acme tenant".to_string()),
        tags: None,
    };

    let admin_policy_response = iam_client.create_policy(admin_policy_request).await?;
    let admin_policy = admin_policy_response.data.unwrap();
    println!(
        "✓ Created tenant admin policy: {}",
        admin_policy.policy_name
    );

    // Create a read-only policy
    let readonly_policy_request = CreatePolicyRequest {
        policy_name: "TenantReadOnlyPolicy".to_string(),
        policy_document: build_tenant_readonly_policy("acme/production"),
        path: Some("/tenant/".to_string()),
        description: Some("Read-only access to production tenant".to_string()),
        tags: None,
    };

    let readonly_policy_response = iam_client.create_policy(readonly_policy_request).await?;
    let readonly_policy = readonly_policy_response.data.unwrap();
    println!(
        "✓ Created read-only policy: {}",
        readonly_policy.policy_name
    );

    // Use these policies for authorization
    let authorizer = TenantAuthorizer::new(vec![
        admin_policy.policy_document,
        readonly_policy.policy_document,
    ]);

    let admin_arn = "arn:aws:iam::123456789012:user/admin";

    // Admin has full access to acme tenant
    let can_delete = authorizer
        .check_permission(admin_arn, "acme/engineering", TenantAction::Delete)
        .await?;
    println!("✓ Admin can delete acme tenant: {}", can_delete);

    let can_read_prod = authorizer
        .check_permission(admin_arn, "acme/production", TenantAction::Read)
        .await?;
    println!("✓ Admin can read production tenant: {}", can_read_prod);

    println!();
    Ok(())
}

/// Example 3: Complex scenario with multiple policies and explicit deny
///
/// This demonstrates advanced policy evaluation including:
/// - Multiple overlapping policies
/// - Explicit deny rules
/// - Wildcard matching
async fn complex_authorization_scenario() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 3: Complex Multi-Policy Scenario\n");

    // Policy 1: Allow all tenant operations
    let allow_all_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": "tenant:*",
            "Resource": "*"
        }]
    }"#;

    // Policy 2: Explicitly deny deletion of production tenants
    let deny_prod_delete_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Deny",
            "Action": "tenant:Delete",
            "Resource": "arn:wami:tenant::acme/production/*"
        }]
    }"#;

    // Policy 3: Allow read-only on all archived tenants
    let archived_readonly_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": "tenant:Read",
            "Resource": "arn:wami:tenant::acme/archived/*"
        }]
    }"#;

    let authorizer = TenantAuthorizer::new(vec![
        allow_all_policy.to_string(),
        deny_prod_delete_policy.to_string(),
        archived_readonly_policy.to_string(),
    ]);

    let user_arn = "arn:aws:iam::123456789012:user/operator";

    // Test different scenarios
    println!("Testing with user: {}\n", user_arn);

    // Can update dev tenants
    let can_update_dev = authorizer
        .check_permission(user_arn, "acme/development/api", TenantAction::Update)
        .await?;
    println!(
        "✓ Can update development tenant: {} (allowed by wildcard policy)",
        can_update_dev
    );

    // Can delete dev tenants
    let can_delete_dev = authorizer
        .check_permission(user_arn, "acme/development/api", TenantAction::Delete)
        .await?;
    println!(
        "✓ Can delete development tenant: {} (allowed by wildcard policy)",
        can_delete_dev
    );

    // Cannot delete production tenants (explicit deny wins)
    let can_delete_prod = authorizer
        .check_permission(user_arn, "acme/production/api", TenantAction::Delete)
        .await?;
    println!(
        "✗ Cannot delete production tenant: {} (explicit deny wins!)",
        !can_delete_prod
    );

    // Can read production tenants
    let can_read_prod = authorizer
        .check_permission(user_arn, "acme/production/api", TenantAction::Read)
        .await?;
    println!(
        "✓ Can read production tenant: {} (allowed by wildcard policy)",
        can_read_prod
    );

    // Can read archived tenants
    let can_read_archived = authorizer
        .check_permission(user_arn, "acme/archived/old-project", TenantAction::Read)
        .await?;
    println!(
        "✓ Can read archived tenant: {} (allowed by archived policy)",
        can_read_archived
    );

    // Cannot update archived tenants
    let can_update_archived = authorizer
        .check_permission(user_arn, "acme/archived/old-project", TenantAction::Update)
        .await?;
    println!(
        "✗ Cannot update archived tenant: {} (not explicitly allowed for archived)",
        !can_update_archived
    );

    println!("\n✅ All examples completed successfully!");
    Ok(())
}
