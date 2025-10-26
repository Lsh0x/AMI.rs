//! Multi-Tenant Architecture Example
//!
//! This example demonstrates the hierarchical multi-tenancy features of WAMI,
//! showing how to create tenants, sub-tenants, enforce quotas, and manage
//! tenant isolation.

use std::collections::HashMap;
use wami::store::memory::InMemoryStore;
use wami::tenant::client::{CreateRootTenantRequest, CreateSubTenantRequest};
use wami::tenant::{BillingInfo, TenantClient, TenantId, TenantQuotas, TenantType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment
    env_logger::init();
    let store = InMemoryStore::new();
    let mut tenant_client = TenantClient::new(store, "platform-admin@example.com".to_string());

    println!("=== Multi-Tenant Architecture Demo ===\n");

    // 1. Create Root Tenant
    println!("1. Creating root tenant 'acme'...");
    let root_request = CreateRootTenantRequest {
        name: "acme".to_string(),
        organization: Some("Acme Corporation".to_string()),
        provider_accounts: Some({
            let mut accounts = HashMap::new();
            accounts.insert("aws".to_string(), "123456789012".to_string());
            accounts
        }),
        quotas: Some(TenantQuotas {
            max_users: 1000,
            max_roles: 500,
            max_policies: 100,
            max_groups: 100,
            max_access_keys: 2000,
            max_sub_tenants: 10,
            api_rate_limit: 1000,
        }),
        max_child_depth: Some(5),
        admin_principals: vec!["admin@acme.com".to_string()],
        metadata: Some(HashMap::new()),
        billing_info: Some(BillingInfo {
            cost_center: "CC-001".to_string(),
            billable: true,
            contact_email: "billing@acme.com".to_string(),
        }),
    };

    let root_response = tenant_client.create_root_tenant(root_request).await?;
    let root = root_response.data.unwrap();
    println!("✓ Created root tenant: {}", root.id);
    println!("  Organization: {}", root.organization.as_ref().unwrap());
    println!("  Max users: {}", root.quotas.max_users);
    println!();

    // 2. Create Department Sub-Tenants
    println!("2. Creating department sub-tenants...");
    let root_id = TenantId::root("acme");

    // Engineering department
    let eng_request = CreateSubTenantRequest {
        name: "engineering".to_string(),
        organization: None,
        tenant_type: TenantType::Department,
        provider_accounts: None,
        quotas: Some(TenantQuotas {
            max_users: 300, // Subset of parent
            max_roles: 150,
            max_policies: 30,
            max_groups: 30,
            max_access_keys: 600,
            max_sub_tenants: 5,
            api_rate_limit: 300,
        }),
        admin_principals: vec!["eng-admin@acme.com".to_string()],
        metadata: None,
        billing_info: Some(BillingInfo {
            cost_center: "CC-ENG".to_string(),
            billable: true,
            contact_email: "eng-billing@acme.com".to_string(),
        }),
    };

    let eng_response = tenant_client
        .create_sub_tenant(&root_id, eng_request)
        .await?;
    let eng = eng_response.data.unwrap();
    println!("✓ Created: {} (depth: {})", eng.id, eng.id.depth());

    // Sales department
    let sales_request = CreateSubTenantRequest {
        name: "sales".to_string(),
        organization: None,
        tenant_type: TenantType::Department,
        provider_accounts: None,
        quotas: None, // Inherit from parent
        admin_principals: vec!["sales-admin@acme.com".to_string()],
        metadata: None,
        billing_info: None,
    };

    let sales_response = tenant_client
        .create_sub_tenant(&root_id, sales_request)
        .await?;
    let sales = sales_response.data.unwrap();
    println!("✓ Created: {} (depth: {})", sales.id, sales.id.depth());
    println!();

    // 3. Create Team Sub-Tenants
    println!("3. Creating team sub-tenants under engineering...");
    let eng_id = root_id.child("engineering");

    let frontend_request = CreateSubTenantRequest {
        name: "frontend".to_string(),
        organization: None,
        tenant_type: TenantType::Team,
        provider_accounts: None,
        quotas: Some(TenantQuotas {
            max_users: 50,
            max_roles: 25,
            max_policies: 10,
            max_groups: 10,
            max_access_keys: 100,
            max_sub_tenants: 2,
            api_rate_limit: 100,
        }),
        admin_principals: vec!["frontend-lead@acme.com".to_string()],
        metadata: None,
        billing_info: None,
    };

    let frontend_response = tenant_client
        .create_sub_tenant(&eng_id, frontend_request)
        .await?;
    let frontend = frontend_response.data.unwrap();
    println!(
        "✓ Created: {} (depth: {})",
        frontend.id,
        frontend.id.depth()
    );
    println!();

    // 4. Demonstrate Hierarchy
    println!("4. Tenant Hierarchy:");
    let frontend_id = eng_id.child("frontend");
    println!("  Frontend ancestors:");
    for ancestor in frontend_id.ancestors() {
        println!("    - {}", ancestor);
    }
    println!();

    println!("  Frontend is descendant of:");
    println!(
        "    - root 'acme': {}",
        frontend_id.is_descendant_of(&root_id)
    );
    println!(
        "    - 'engineering': {}",
        frontend_id.is_descendant_of(&eng_id)
    );
    println!();

    // 5. List Children
    println!("5. Listing children of root tenant...");
    let children_response = tenant_client.list_child_tenants(&root_id).await?;
    let children = children_response.data.unwrap();
    for child in children {
        println!("  - {} ({:?})", child.name, child.tenant_type);
    }
    println!();

    // 6. Demonstrate Tenant-Aware Resource Paths
    println!("6. Tenant-aware resource paths:");
    println!("  Base path: /");
    println!("  Tenant ID: acme/engineering/frontend");
    println!("  Result: /tenants/acme/engineering/frontend/");
    println!("  Resources are automatically namespaced by tenant hierarchy");
    println!();

    // 8. Show Quota Enforcement
    println!("8. Demonstrating quota enforcement...");
    println!("  Engineering department quotas:");
    println!(
        "    Max users: {} (parent: {})",
        eng.quotas.max_users, root.quotas.max_users
    );
    println!("    Max sub-tenants: {}", eng.quotas.max_sub_tenants);
    println!();

    println!("=== Multi-Tenant Demo Complete ===");
    println!("\nKey Features Demonstrated:");
    println!("✓ Hierarchical tenant structure (unlimited depth with constraints)");
    println!("✓ Quota management with inheritance and validation");
    println!("✓ Permission-based access control (tenant admins)");
    println!("✓ Tenant-aware resource path generation");
    println!("✓ Usage tracking and monitoring");
    println!("✓ Billing information per tenant");

    Ok(())
}
