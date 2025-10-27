//! Unified Store Architecture Demo
//!
//! This example demonstrates the new ARN-centric unified store architecture.
//!
//! # Key Features
//!
//! 1. **Single Store Interface**: One trait for all resource types
//! 2. **ARN-Based Operations**: Get, query, put, delete by ARN
//! 3. **Wildcard Queries**: Powerful pattern matching across resources
//! 4. **Tenant Isolation**: Opaque tenant hashing for security
//! 5. **Type Safety**: Resource enum with safe downcasting
//!
//! # Run This Example
//!
//! ```bash
//! cargo run --example unified_store_demo
//! ```

use chrono::Utc;
use wami::error::Result;
use wami::iam::policy::Policy;
use wami::iam::role::Role;
use wami::iam::user::User;
use wami::store::memory::UnifiedInMemoryStore;
use wami::store::resource::Resource;
use wami::store::traits::Store;

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> Result<()> {
    println!("=== WAMI Unified Store Architecture Demo ===\n");

    // ========================================
    // Part 1: Store Initialization
    // ========================================
    println!("📦 Part 1: Creating Unified Store");
    println!("   Single HashMap for all resources\n");

    let store = UnifiedInMemoryStore::new();

    // ========================================
    // Part 2: Multi-Tenant Resource Creation
    // ========================================
    println!("🏢 Part 2: Multi-Tenant Resources");
    println!("   Creating resources in different tenants\n");

    // Tenant A (Production)
    let tenant_a_hash = "prod-a1b2c3"; // SHA-256 hash of real tenant ID

    let alice = User {
        arn: format!("arn:wami:iam:{}:user/alice", tenant_a_hash),
        user_name: "alice".to_string(),
        user_id: "AIDAALICE".to_string(),
        path: "/".to_string(),
        create_date: Utc::now(),
        password_last_used: None,
        permissions_boundary: None,
        tags: Vec::new(),
        wami_arn: format!("arn:wami:iam:{}:user/alice", tenant_a_hash),
        providers: Vec::new(),
        tenant_id: None,
    };

    let admin_role = Role {
        arn: format!("arn:wami:iam:{}:role/Admin", tenant_a_hash),
        role_name: "Admin".to_string(),
        role_id: "RIDAADMIN".to_string(),
        path: "/".to_string(),
        create_date: Utc::now(),
        assume_role_policy_document: "{}".to_string(),
        description: Some("Administrator role".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: Vec::new(),
        wami_arn: format!("arn:wami:iam:{}:role/Admin", tenant_a_hash),
        providers: Vec::new(),
        tenant_id: None,
    };

    // Tenant B (Staging)
    let tenant_b_hash = "stag-xyz789"; // SHA-256 hash of different tenant ID

    let bob = User {
        arn: format!("arn:wami:iam:{}:user/bob", tenant_b_hash),
        user_name: "bob".to_string(),
        user_id: "AIDABOB".to_string(),
        path: "/".to_string(),
        create_date: Utc::now(),
        password_last_used: None,
        permissions_boundary: None,
        tags: Vec::new(),
        wami_arn: format!("arn:wami:iam:{}:user/bob", tenant_b_hash),
        providers: Vec::new(),
        tenant_id: None,
    };

    let dev_role = Role {
        arn: format!("arn:wami:iam:{}:role/Developer", tenant_b_hash),
        role_name: "Developer".to_string(),
        role_id: "RIDADEV".to_string(),
        path: "/".to_string(),
        create_date: Utc::now(),
        assume_role_policy_document: "{}".to_string(),
        description: Some("Developer role".to_string()),
        max_session_duration: Some(7200),
        permissions_boundary: None,
        tags: Vec::new(),
        wami_arn: format!("arn:wami:iam:{}:role/Developer", tenant_b_hash),
        providers: Vec::new(),
        tenant_id: None,
    };

    // Global (Cross-Tenant Policy)
    let global_policy = Policy {
        arn: "arn:wami:iam:global:policy/ReadOnly".to_string(),
        policy_name: "ReadOnly".to_string(),
        policy_id: "PIDAREADONLY".to_string(),
        path: "/".to_string(),
        default_version_id: "v1".to_string(),
        policy_document: r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":"*:Get*","Resource":"*"}]}"#.to_string(),
        attachment_count: 0,
        permissions_boundary_usage_count: 0,
        is_attachable: true,
        description: Some("Global read-only policy".to_string()),
        create_date: Utc::now(),
        update_date: Utc::now(),
        tags: Vec::new(),
        wami_arn: "arn:wami:iam:global:policy/ReadOnly".to_string(),
        providers: Vec::new(),
        tenant_id: None,
    };

    // Store all resources
    store.put(Resource::User(alice.clone())).await?;
    store.put(Resource::Role(admin_role.clone())).await?;
    store.put(Resource::User(bob.clone())).await?;
    store.put(Resource::Role(dev_role.clone())).await?;
    store.put(Resource::Policy(global_policy.clone())).await?;

    println!("   ✓ Created 2 users (Alice in prod, Bob in staging)");
    println!("   ✓ Created 2 roles (Admin in prod, Developer in staging)");
    println!("   ✓ Created 1 global policy\n");

    // ========================================
    // Part 3: Get by Exact ARN
    // ========================================
    println!("🔍 Part 3: Get Resource by Exact ARN");

    let alice_arn = format!("arn:wami:iam:{}:user/alice", tenant_a_hash);
    if let Some(resource) = store.get(&alice_arn).await? {
        if let Some(user) = resource.as_user() {
            println!("   Found user: {} (ID: {})", user.user_name, user.user_id);
            println!("   ARN: {}", user.arn);
            println!("   ✓ Tenant hash is opaque: {}\n", tenant_a_hash);
        }
    }

    // ========================================
    // Part 4: Wildcard Queries
    // ========================================
    println!("🔎 Part 4: Wildcard Queries");

    // Query 1: All users in Production tenant
    let prod_users = store
        .query(&format!("arn:wami:iam:{}:user/*", tenant_a_hash))
        .await?;
    println!("   Query: All users in Production tenant");
    println!("   Pattern: arn:wami:iam:{}:user/*", tenant_a_hash);
    println!("   Results: {} user(s)", prod_users.len());

    // Query 2: All resources in Staging tenant
    let stag_resources = store
        .query(&format!("arn:wami:iam:{}:*", tenant_b_hash))
        .await?;
    println!("\n   Query: All resources in Staging tenant");
    println!("   Pattern: arn:wami:iam:{}:*", tenant_b_hash);
    println!("   Results: {} resource(s)", stag_resources.len());

    // Query 3: All users across all tenants
    let all_users = store.query("arn:wami:iam:*:user/*").await?;
    println!("\n   Query: All users across tenants");
    println!("   Pattern: arn:wami:iam:*:user/*");
    println!("   Results: {} user(s)", all_users.len());
    for res in &all_users {
        if let Some(user) = res.as_user() {
            println!("      - {} ({})", user.user_name, user.arn);
        }
    }

    // Query 4: All roles across all tenants
    let all_roles = store.query("arn:wami:iam:*:role/*").await?;
    println!("\n   Query: All roles across tenants");
    println!("   Pattern: arn:wami:iam:*:role/*");
    println!("   Results: {} role(s)", all_roles.len());
    for res in &all_roles {
        if let Some(role) = res.as_role() {
            println!("      - {} ({})", role.role_name, role.arn);
        }
    }

    // Query 5: Global resources
    let global_resources = store.query("arn:wami:iam:global:*").await?;
    println!("\n   Query: Global resources");
    println!("   Pattern: arn:wami:iam:global:*");
    println!("   Results: {} resource(s)\n", global_resources.len());

    // ========================================
    // Part 5: Tenant-Scoped Operations
    // ========================================
    println!("🏗️  Part 5: Tenant-Scoped Operations");

    let prod_count = store.count_tenant(tenant_a_hash).await?;
    let stag_count = store.count_tenant(tenant_b_hash).await?;

    println!("   Production tenant: {} resources", prod_count);
    println!("   Staging tenant: {} resources", stag_count);

    // List all resources in Production
    let prod_resources = store.list_tenant_resources(tenant_a_hash).await?;
    println!("\n   Production tenant resources:");
    for res in &prod_resources {
        match res {
            Resource::User(u) => println!("      - User: {}", u.user_name),
            Resource::Role(r) => println!("      - Role: {}", r.role_name),
            Resource::Policy(p) => println!("      - Policy: {}", p.policy_name),
            _ => println!("      - Other resource"),
        }
    }

    println!();

    // ========================================
    // Part 6: Resource Type Filtering
    // ========================================
    println!("📋 Part 6: Resource Type Filtering");

    let prod_users_typed = store.list_by_type(tenant_a_hash, "user").await?;
    let prod_roles_typed = store.list_by_type(tenant_a_hash, "role").await?;

    println!("   Production Users: {}", prod_users_typed.len());
    println!("   Production Roles: {}", prod_roles_typed.len());

    let all_users_typed = store.list_by_type_global("user").await?;
    let all_roles_typed = store.list_by_type_global("role").await?;

    println!("   Global Users: {}", all_users_typed.len());
    println!("   Global Roles: {}\n", all_roles_typed.len());

    // ========================================
    // Part 7: Update Operations
    // ========================================
    println!("✏️  Part 7: Update Operations");

    // Update Alice's user
    let mut alice_updated = alice.clone();
    alice_updated.password_last_used = Some(Utc::now());

    store.put(Resource::User(alice_updated)).await?;
    println!("   ✓ Updated Alice's password_last_used timestamp\n");

    // ========================================
    // Part 8: Bulk Operations
    // ========================================
    println!("📦 Part 8: Bulk Operations");

    // Create multiple new users
    let new_users = vec![
        Resource::User(User {
            arn: format!("arn:wami:iam:{}:user/charlie", tenant_a_hash),
            user_name: "charlie".to_string(),
            user_id: "AIDACHARLIE".to_string(),
            path: "/".to_string(),
            create_date: Utc::now(),
            password_last_used: None,
            permissions_boundary: None,
            tags: Vec::new(),
            wami_arn: format!("arn:wami:iam:{}:user/charlie", tenant_a_hash),
            providers: Vec::new(),
            tenant_id: None,
        }),
        Resource::User(User {
            arn: format!("arn:wami:iam:{}:user/diana", tenant_a_hash),
            user_name: "diana".to_string(),
            user_id: "AIDADIANA".to_string(),
            path: "/".to_string(),
            create_date: Utc::now(),
            password_last_used: None,
            permissions_boundary: None,
            tags: Vec::new(),
            wami_arn: format!("arn:wami:iam:{}:user/diana", tenant_a_hash),
            providers: Vec::new(),
            tenant_id: None,
        }),
    ];

    let count = store.put_batch(new_users).await?;
    println!("   ✓ Bulk created {} users\n", count);

    // ========================================
    // Part 9: Delete Operations
    // ========================================
    println!("🗑️  Part 9: Delete Operations");

    let charlie_arn = format!("arn:wami:iam:{}:user/charlie", tenant_a_hash);
    let deleted = store.delete(&charlie_arn).await?;
    println!("   ✓ Deleted user charlie: {}", deleted);

    // Try to delete again (should return false)
    let deleted_again = store.delete(&charlie_arn).await?;
    println!(
        "   ✓ Delete again returned: {} (expected false)\n",
        deleted_again
    );

    // ========================================
    // Part 10: Final Statistics
    // ========================================
    println!("📊 Part 10: Final Statistics");

    let total_resources = store.count_all().await?;
    let prod_final = store.count_tenant(tenant_a_hash).await?;
    let stag_final = store.count_tenant(tenant_b_hash).await?;

    println!("   Total Resources: {}", total_resources);
    println!("   Production: {} resources", prod_final);
    println!("   Staging: {} resources", stag_final);

    // Resource type breakdown
    let total_users = store.list_by_type_global("user").await?;
    let total_roles = store.list_by_type_global("role").await?;
    let total_policies = store.list_by_type_global("policy").await?;

    println!("\n   By Type:");
    println!("   - Users: {}", total_users.len());
    println!("   - Roles: {}", total_roles.len());
    println!("   - Policies: {}", total_policies.len());

    // ========================================
    // Summary
    // ========================================
    println!("\n=== Key Advantages ===");
    println!("✓ Single unified interface for all resources");
    println!("✓ ARN-based operations with tenant isolation");
    println!("✓ Opaque tenant hashing for security");
    println!("✓ Powerful wildcard queries");
    println!("✓ Type-safe resource handling");
    println!("✓ O(1) lookups by ARN");
    println!("✓ Simple implementation (single HashMap)");
    println!("\n=== Demo Complete ===\n");

    Ok(())
}
