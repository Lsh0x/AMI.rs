//! Simple Multi-Tenant Setup
//!
//! This example demonstrates:
//! - Creating multiple tenants
//! - Creating users within specific tenants
//! - Tenant isolation (resources belong to specific tenants)
//!
//! Scenario: Two companies (CompanyA and CompanyB) each with their own users.
//!
//! Run with: `cargo run --example 04_simple_multi_tenant`

use std::sync::{Arc, RwLock};
use wami::provider::AwsProvider;
use wami::service::{TenantService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::{CreateUserRequest, ListUsersRequest};
use wami::wami::tenant::model::TenantId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple Multi-Tenant Setup ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    let _provider = Arc::new(AwsProvider::new());

    // === CREATE TENANTS ===
    println!("Step 1: Creating tenants...\n");

    let tenant_service = TenantService::new(store.clone(), "root".to_string());

    // Create Company A tenant
    let _company_a_id = TenantId::new("company-a");
    tenant_service
        .create_tenant(
            "company-a".to_string(),
            Some("Company A Corp".to_string()),
            None, // No parent, this is a root tenant
        )
        .await?;
    println!("✓ Created tenant: company-a");

    // Create Company B tenant
    let _company_b_id = TenantId::new("company-b");
    tenant_service
        .create_tenant(
            "company-b".to_string(),
            Some("Company B Inc".to_string()),
            None,
        )
        .await?;
    println!("✓ Created tenant: company-b");

    // === CREATE USERS IN EACH TENANT ===
    println!("\nStep 2: Creating users in each tenant...\n");

    // Company A users (using company-a account ID)
    println!("Creating users for Company A...");
    let user_service_a = UserService::new(
        store.clone(),
        "company-a".to_string(), // Tenant-specific account ID
    );

    let alice_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/company-a/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let alice = user_service_a.create_user(alice_req).await?;
    println!("✓ Created alice in company-a");
    println!("  ARN: {}", alice.arn);

    let bob_req = CreateUserRequest {
        user_name: "bob".to_string(),
        path: Some("/company-a/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    user_service_a.create_user(bob_req).await?;
    println!("✓ Created bob in company-a");

    // Company B users (using company-b account ID)
    println!("\nCreating users for Company B...");
    let user_service_b = UserService::new(
        store.clone(),
        "company-b".to_string(), // Tenant-specific account ID
    );

    let charlie_req = CreateUserRequest {
        user_name: "charlie".to_string(),
        path: Some("/company-b/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let charlie = user_service_b.create_user(charlie_req).await?;
    println!("✓ Created charlie in company-b");
    println!("  ARN: {}", charlie.arn);

    let diana_req = CreateUserRequest {
        user_name: "diana".to_string(),
        path: Some("/company-b/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    user_service_b.create_user(diana_req).await?;
    println!("✓ Created diana in company-b");

    // === DEMONSTRATE ISOLATION ===
    println!("\n\nStep 3: Demonstrating tenant isolation...\n");

    // List all users (cross-tenant view - usually restricted in production)
    let all_users_service = UserService::new(store.clone(), "root".to_string());
    let (all_users, _, _) = all_users_service
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!("Total users across all tenants: {}", all_users.len());

    // Company A can only see its users
    let (company_a_users, _, _) = user_service_a
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!(
        "\nCompany A users (via company-a service): {}",
        company_a_users.len()
    );
    for user in &company_a_users {
        println!("  - {} (path: {})", user.user_name, user.path);
    }

    // Company B can only see its users
    let (company_b_users, _, _) = user_service_b
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!(
        "\nCompany B users (via company-b service): {}",
        company_b_users.len()
    );
    for user in &company_b_users {
        println!("  - {} (path: {})", user.user_name, user.path);
    }

    // List all tenants
    println!("\n\nStep 4: Listing all tenants...\n");
    let all_tenants = tenant_service.list_tenants().await?;
    println!("✓ Found {} tenants:", all_tenants.len());
    for tenant in &all_tenants {
        println!("  - {} ({})", tenant.name, tenant.id);
    }

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Each tenant has its own namespace for resources");
    println!("- Services can be scoped to specific tenants via account_id");
    println!("- ARNs reflect the tenant/account ownership");
    println!("- Tenant isolation ensures data security in multi-tenant apps");

    Ok(())
}
