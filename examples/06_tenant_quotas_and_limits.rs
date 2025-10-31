//! Tenant Quotas and Limits
//!
//! This example demonstrates:
//! - Setting resource quotas on tenants
//! - Quota inheritance in hierarchies
//! - Getting effective quotas for a tenant
//!
//! Scenario: Managing resource limits for different organizational levels.
//!
//! Run with: `cargo run --example 06_tenant_quotas_and_limits`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::TenantService;
use wami::store::memory::InMemoryWamiStore;
use wami::wami::tenant::model::{QuotaMode, TenantId, TenantQuotas};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Tenant Quotas and Limits ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create root context
    let root_context = WamiContext::builder()
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
        .is_root(true)
        .build()?;

    let tenant_service = TenantService::new(store.clone());

    // === CREATE TENANTS WITH QUOTAS ===
    println!("Step 1: Creating tenants with quota configurations...\n");

    // Root tenant with high quotas
    let root_id = TenantId::new("company");
    let root_tenant = tenant_service
        .create_tenant(
            &root_context,
            "company".to_string(),
            Some("Company Root".to_string()),
            None,
        )
        .await?;

    // Update quotas after creation
    let mut updated_root = root_tenant.clone();
    updated_root.quotas = TenantQuotas {
        max_users: 1000,
        max_groups: 100,
        max_roles: 200,
        max_policies: 500,
        max_access_keys: 2000,
        max_sub_tenants: 50,
        api_rate_limit: 10000,
    };
    updated_root.quota_mode = QuotaMode::Inherited;
    tenant_service.update_tenant(updated_root).await?;
    println!("✓ Created root tenant with quotas:");
    println!("  - Max users: 1000");
    println!("  - Max groups: 100");
    println!("  - Max roles: 200");
    println!("  - Max policies: 500");

    // Department with lower quotas
    let dept_id = root_id.child("engineering");
    let dept_tenant = tenant_service
        .create_tenant(
            &root_context,
            "engineering".to_string(),
            Some("Engineering Dept".to_string()),
            Some(root_id.clone()),
        )
        .await?;

    let mut updated_dept = dept_tenant.clone();
    updated_dept.quotas = TenantQuotas {
        max_users: 100,
        max_groups: 10,
        max_roles: 20,
        max_policies: 50,
        max_access_keys: 200,
        max_sub_tenants: 10,
        api_rate_limit: 1000,
    };
    updated_dept.quota_mode = QuotaMode::Inherited;
    tenant_service.update_tenant(updated_dept).await?;
    println!("\n✓ Created engineering tenant with reduced quotas:");
    println!("  - Max users: 100");
    println!("  - Max groups: 10");

    // Team with inherited quotas
    let team_id = dept_id.child("backend-team");
    let team_tenant = tenant_service
        .create_tenant(
            &root_context,
            "backend-team".to_string(),
            Some("Backend Team".to_string()),
            Some(dept_id.clone()),
        )
        .await?;

    let mut updated_team = team_tenant.clone();
    updated_team.quotas = TenantQuotas {
        max_users: 20,
        max_groups: 5,
        max_roles: 10,
        max_policies: 15,
        max_access_keys: 40,
        max_sub_tenants: 2,
        api_rate_limit: 200,
    };
    updated_team.quota_mode = QuotaMode::Inherited;
    tenant_service.update_tenant(updated_team).await?;
    println!("\n✓ Created backend-team with even lower quotas:");
    println!("  - Max users: 20");
    println!("  - Max groups: 5");

    // === QUERY EFFECTIVE QUOTAS ===
    println!("\n\nStep 2: Querying effective quotas...\n");

    // Root tenant effective quotas
    let root_effective = tenant_service.get_effective_quotas(&root_id).await?;
    println!("Root tenant effective quotas:");
    println!("  - Max users: {}", root_effective.max_users);
    println!("  - Max groups: {}", root_effective.max_groups);

    // Department effective quotas (should be inherited/limited by parent)
    let dept_effective = tenant_service.get_effective_quotas(&dept_id).await?;
    println!("\nEngineering dept effective quotas:");
    println!("  - Max users: {}", dept_effective.max_users);
    println!("  - Max groups: {}", dept_effective.max_groups);

    // Team effective quotas
    let team_effective = tenant_service.get_effective_quotas(&team_id).await?;
    println!("\nBackend team effective quotas:");
    println!("  - Max users: {}", team_effective.max_users);
    println!("  - Max groups: {}", team_effective.max_groups);

    // === DEMONSTRATE USAGE TRACKING ===
    println!("\n\nStep 3: Understanding quota usage...\n");

    println!("Note: In production, you would:");
    println!("- Track actual resource usage per tenant");
    println!("- Enforce quotas when creating resources");
    println!("- Return errors when quotas are exceeded");
    println!("- Monitor quota utilization for capacity planning");

    println!("\nExample quota enforcement logic:");
    println!("```rust");
    println!("let usage = tenant_service.get_tenant_usage(&tenant_id).await?;");
    println!("let quotas = tenant_service.get_effective_quotas(&tenant_id).await?;");
    println!("if usage.user_count >= quotas.max_users {{");
    println!("    return Err(\"Quota exceeded: max users reached\");");
    println!("}}");
    println!("```");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Quotas prevent resource exhaustion in multi-tenant systems");
    println!("- Hierarchical quotas enable delegation with limits");
    println!("- Effective quotas consider the entire tenant hierarchy");
    println!("- Quota enforcement happens at resource creation time");

    Ok(())
}
