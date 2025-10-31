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
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{TenantService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::{CreateUserRequest, ListUsersRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple Multi-Tenant Setup ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create root context
    let root_context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single(0)) // Root tenant ID is 0
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

    // === CREATE TENANTS ===
    println!("Step 1: Creating tenants...\n");

    let tenant_service = TenantService::new(store.clone());

    // Create Company A tenant (service generates numeric ID automatically)
    let company_a = tenant_service
        .create_tenant(
            &root_context,
            "company-a".to_string(),
            Some("Company A Corp".to_string()),
            None, // No parent, this is a root tenant
        )
        .await?;
    let company_a_id = company_a.id.clone();
    println!("✓ Created tenant: company-a (ID: {})", company_a.id);

    // Create Company B tenant (service generates numeric ID automatically)
    let company_b = tenant_service
        .create_tenant(
            &root_context,
            "company-b".to_string(),
            Some("Company B Inc".to_string()),
            None,
        )
        .await?;
    let company_b_id = company_b.id.clone();
    println!("✓ Created tenant: company-b (ID: {})", company_b.id);

    // === CREATE USERS IN EACH TENANT ===
    println!("\nStep 2: Creating users in each tenant...\n");

    // Get numeric tenant IDs for contexts (first segment is the root tenant ID)
    let company_a_tenant_id = company_a_id.segments()[0];
    let company_b_tenant_id = company_b_id.segments()[0];

    // Company A context
    let company_a_context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single(company_a_tenant_id))
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(company_a_tenant_id))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Company B context
    let company_b_context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single(company_b_tenant_id))
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(company_b_tenant_id))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Company A users
    println!("Creating users for Company A...");
    let user_service = UserService::new(store.clone());

    let alice_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/company-a/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let alice = user_service
        .create_user(&company_a_context, alice_req)
        .await?;
    println!("✓ Created alice in company-a");
    println!("  ARN: {}", alice.arn);

    let bob_req = CreateUserRequest {
        user_name: "bob".to_string(),
        path: Some("/company-a/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    user_service
        .create_user(&company_a_context, bob_req)
        .await?;
    println!("✓ Created bob in company-a");

    // Company B users
    println!("\nCreating users for Company B...");

    let charlie_req = CreateUserRequest {
        user_name: "charlie".to_string(),
        path: Some("/company-b/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let charlie = user_service
        .create_user(&company_b_context, charlie_req)
        .await?;
    println!("✓ Created charlie in company-b");
    println!("  ARN: {}", charlie.arn);

    let diana_req = CreateUserRequest {
        user_name: "diana".to_string(),
        path: Some("/company-b/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    user_service
        .create_user(&company_b_context, diana_req)
        .await?;
    println!("✓ Created diana in company-b");

    // === DEMONSTRATE ISOLATION ===
    println!("\n\nStep 3: Demonstrating tenant isolation...\n");

    // List all users (cross-tenant view - usually restricted in production)
    let (all_users, _, _) = user_service
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!("Total users across all tenants: {}", all_users.len());

    // Company A can only see its users (using company-a context)
    let (company_a_users, _, _) = user_service
        .list_users(ListUsersRequest {
            path_prefix: Some("/company-a/".to_string()),
            pagination: None,
        })
        .await?;
    println!(
        "\nCompany A users (filtered by path): {}",
        company_a_users.len()
    );
    for user in &company_a_users {
        println!("  - {} (path: {})", user.user_name, user.path);
    }

    // Company B can only see its users (using company-b context)
    let (company_b_users, _, _) = user_service
        .list_users(ListUsersRequest {
            path_prefix: Some("/company-b/".to_string()),
            pagination: None,
        })
        .await?;
    println!(
        "\nCompany B users (filtered by path): {}",
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
    println!("- WamiContext is used to scope operations to specific tenants");
    println!("- ARNs reflect the tenant ownership via tenant_path");
    println!("- Tenant isolation ensures data security in multi-tenant apps");

    Ok(())
}
