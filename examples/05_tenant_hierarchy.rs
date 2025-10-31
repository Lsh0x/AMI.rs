//! Tenant Hierarchy
//!
//! This example demonstrates:
//! - Creating nested tenant structures (root → department → team)
//! - Hierarchical tenant IDs
//! - Querying tenant ancestors and descendants
//!
//! Scenario: An organization with departments and teams.
//!
//! Run with: `cargo run --example 05_tenant_hierarchy`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::TenantService;
use wami::store::memory::InMemoryWamiStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Tenant Hierarchy ===\n");

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

    let tenant_service = TenantService::new(store.clone());

    // === CREATE HIERARCHICAL TENANTS ===
    println!("Step 1: Creating hierarchical tenant structure...\n");

    // Root: acme-corp
    println!("Creating root tenant: acme-corp");
    let root_tenant = tenant_service
        .create_tenant(
            &root_context,
            "acme-corp".to_string(),
            Some("ACME Corporation".to_string()),
            None,
        )
        .await?;
    let root_id = root_tenant.id.clone();
    println!("✓ Created: acme-corp (ID: {})", root_id);

    // Department level: engineering, sales
    println!("\nCreating department tenants...");
    let eng_tenant = tenant_service
        .create_tenant(
            &root_context,
            "engineering".to_string(),
            Some("Engineering Department".to_string()),
            Some(root_id.clone()),
        )
        .await?;
    let eng_id = eng_tenant.id.clone();
    println!("✓ Created: acme-corp/engineering (ID: {})", eng_id);

    let sales_tenant = tenant_service
        .create_tenant(
            &root_context,
            "sales".to_string(),
            Some("Sales Department".to_string()),
            Some(root_id.clone()),
        )
        .await?;
    let _sales_id = sales_tenant.id.clone();
    println!("✓ Created: acme-corp/sales (ID: {})", _sales_id);

    // Team level under engineering: backend, frontend
    println!("\nCreating team tenants under engineering...");
    let backend_tenant = tenant_service
        .create_tenant(
            &root_context,
            "backend".to_string(),
            Some("Backend Team".to_string()),
            Some(eng_id.clone()),
        )
        .await?;
    let backend_id = backend_tenant.id.clone();
    println!(
        "✓ Created: acme-corp/engineering/backend (ID: {})",
        backend_id
    );

    let frontend_tenant = tenant_service
        .create_tenant(
            &root_context,
            "frontend".to_string(),
            Some("Frontend Team".to_string()),
            Some(eng_id.clone()),
        )
        .await?;
    let _frontend_id = frontend_tenant.id.clone();
    println!(
        "✓ Created: acme-corp/engineering/frontend (ID: {})",
        _frontend_id
    );

    // === QUERY HIERARCHY ===
    println!("\n\nStep 2: Querying tenant hierarchy...\n");

    // List all tenants
    let all_tenants = tenant_service.list_tenants().await?;
    println!("Total tenants created: {}", all_tenants.len());
    for tenant in &all_tenants {
        println!("  - {} (ID: {})", tenant.name, tenant.id);
    }

    // Get ancestors of backend team
    println!("\nAncestors of 'backend' team:");
    let backend_ancestors = tenant_service.get_ancestors(&backend_id).await?;
    println!("  Found {} ancestors", backend_ancestors.len());
    for ancestor in &backend_ancestors {
        println!("    ← {} ({})", ancestor.name, ancestor.id);
    }

    // Get children of engineering department
    println!("\nChildren of 'engineering' department:");
    let eng_children = tenant_service.list_child_tenants(&eng_id).await?;
    println!("  Found {} children", eng_children.len());
    for child in &eng_children {
        println!("    → {}", child.name);
    }

    // Get all descendants of root
    println!("\nAll descendants of 'acme-corp' (recursive):");
    let root_descendants = tenant_service.get_descendants(&root_id).await?;
    println!("  Found {} descendants", root_descendants.len());
    for descendant in &root_descendants {
        println!("    → {}", descendant);
    }

    // === DEMONSTRATE HIERARCHY BENEFITS ===
    println!("\n\nStep 3: Understanding hierarchy benefits...\n");

    println!("Hierarchy visualization:");
    println!("acme-corp (root)");
    println!("├── engineering");
    println!("│   ├── backend");
    println!("│   └── frontend");
    println!("└── sales");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- TenantId supports hierarchical structures with .child()");
    println!("- Ancestors represent the path from tenant to root");
    println!("- Descendants include all child tenants recursively");
    println!("- Hierarchies enable organizational modeling and quota inheritance");

    Ok(())
}
