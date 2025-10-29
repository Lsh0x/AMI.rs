# WAMI Examples

Complete working examples for common use cases.

## Table of Contents

- [Basic Examples](#basic-examples)
- [Advanced Examples](#advanced-examples)
- [Real-world Scenarios](#real-world-scenarios)

---

## Basic Examples

### Example 1: User Management System

Complete user CRUD operations:

```rust
use wami::wami::identity::user;
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::UserStore;
use wami::provider::aws::AwsProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let account = "123456789012";
    
    // Create users
    for name in ["alice", "bob", "charlie"] {
        let user = user::builder::build_user(
            name.to_string(),
            Some("/engineering/".to_string()),
            &provider,
            account
        );
        store.create_user(user).await?;
        println!("✅ Created: {}", name);
    }
    
    // List all users
    let (users, _, _) = store.list_users(None, None).await?;
    println!("\n📋 Total users: {}", users.len());
    
    // Update Alice's path
    if let Some(mut alice) = store.get_user("alice").await? {
        alice.path = Some("/admin/".to_string());
        store.update_user(alice).await?;
        println!("✅ Updated Alice's path");
    }
    
    // Delete Bob
    store.delete_user("bob").await?;
    println!("✅ Deleted Bob");
    
    // Final count
    let (final_users, _, _) = store.list_users(None, None).await?;
    println!("\n📊 Final count: {} users", final_users.len());
    
    Ok(())
}
```

### Example 2: Group Management and Membership

```rust
use wami::wami::identity::{user, group};
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::{UserStore, GroupStore};
use wami::provider::aws::AwsProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let account = "123456789012";
    
    // Create groups
    for group_name in ["admins", "developers", "viewers"] {
        let grp = group::builder::build_group(
            group_name.to_string(),
            Some("/".to_string()),
            &provider,
            account
        );
        store.create_group(grp).await?;
    }
    println!("✅ Created 3 groups");
    
    // Create users
    for user_name in ["alice", "bob"] {
        let usr = user::builder::build_user(
            user_name.to_string(),
            None,
            &provider,
            account
        );
        store.create_user(usr).await?;
    }
    println!("✅ Created 2 users");
    
    // Add users to groups
    store.add_user_to_group("admins", "alice").await?;
    store.add_user_to_group("developers", "alice").await?;
    store.add_user_to_group("developers", "bob").await?;
    store.add_user_to_group("viewers", "bob").await?;
    
    // List Alice's groups
    let alice_groups = store.list_groups_for_user("alice").await?;
    println!("\n👤 Alice is in {} groups:", alice_groups.len());
    for g in alice_groups {
        println!("   - {}", g.group_name);
    }
    
    // List Bob's groups
    let bob_groups = store.list_groups_for_user("bob").await?;
    println!("\n👤 Bob is in {} groups:", bob_groups.len());
    for g in bob_groups {
        println!("   - {}", g.group_name);
    }
    
    Ok(())
}
```

### Example 3: Access Keys and Credentials

```rust
use wami::wami::identity::user;
use wami::wami::credentials::access_key;
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::{UserStore, AccessKeyStore};
use wami::provider::aws::AwsProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let account = "123456789012";
    
    // Create user
    let user = user::builder::build_user(
        "service-account".to_string(),
        Some("/services/".to_string()),
        &provider,
        account
    );
    store.create_user(user).await?;
    
    // Create 2 access keys
    for i in 1..=2 {
        let key = access_key::builder::build_access_key(
            "service-account".to_string(),
            &provider,
            account
        );
        let created = store.create_access_key(key).await?;
        println!("\n🔑 Access Key #{}:", i);
        println!("   ID: {}", created.access_key_id);
        println!("   Secret: {}", created.secret_access_key);
        println!("   Status: {:?}", created.status);
    }
    
    // List all keys
    let keys = store.list_access_keys("service-account").await?;
    println!("\n📊 Total keys for service-account: {}", keys.len());
    
    Ok(())
}
```

### Example 4: Role with Trust Policy

```rust
use wami::wami::identity::role;
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::RoleStore;
use wami::provider::aws::AwsProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    let account = "123456789012";
    
    // Lambda execution role
    let lambda_trust = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": {"Service": "lambda.amazonaws.com"},
            "Action": "sts:AssumeRole"
        }]
    }"#;
    
    let lambda_role = role::builder::build_role(
        "LambdaExecutionRole".to_string(),
        lambda_trust.to_string(),
        Some("/service-roles/".to_string()),
        Some("Allows Lambda to call AWS services".to_string()),
        None,
        &provider,
        account
    );
    let created = store.create_role(lambda_role).await?;
    println!("✅ Lambda role: {}", created.arn);
    
    // EC2 instance role
    let ec2_trust = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": {"Service": "ec2.amazonaws.com"},
            "Action": "sts:AssumeRole"
        }]
    }"#;
    
    let ec2_role = role::builder::build_role(
        "EC2InstanceRole".to_string(),
        ec2_trust.to_string(),
        Some("/service-roles/".to_string()),
        Some("Allows EC2 instances to call AWS services".to_string()),
        None,
        &provider,
        account
    );
    let created = store.create_role(ec2_role).await?;
    println!("✅ EC2 role: {}", created.arn);
    
    Ok(())
}
```

---

## Advanced Examples

### Example 5: Multi-tenant Organization

```rust
use wami::wami::tenant::{Tenant, TenantId, TenantQuotas, TenantStatus, TenantType, QuotaMode};
use wami::store::memory::InMemoryTenantStore;
use wami::store::traits::TenantStore;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = InMemoryTenantStore::new();
    
    // Root tenant
    let root_id = TenantId::root("acme-corp");
    let root = Tenant {
        id: root_id.clone(),
        name: "Acme Corporation".to_string(),
        parent_id: None,
        organization: Some("Acme Inc.".to_string()),
        tenant_type: TenantType::Root,
        provider_accounts: HashMap::new(),
        arn: "arn:wami:tenant::acme-corp".to_string(),
        providers: vec![],
        created_at: chrono::Utc::now(),
        status: TenantStatus::Active,
        quotas: TenantQuotas {
            max_users: 1000,
            max_roles: 500,
            max_policies: 200,
            max_groups: 100,
            max_access_keys: 2000,
            max_sub_tenants: 10,
            api_rate_limit: 1000,
        },
        quota_mode: QuotaMode::Override,
        max_child_depth: 5,
        can_create_sub_tenants: true,
        admin_principals: vec![],
        metadata: HashMap::new(),
        billing_info: None,
    };
    store.create_tenant(root).await?;
    println!("✅ Created root tenant: acme-corp");
    
    // Engineering department
    let eng_id = root_id.child("engineering");
    let eng = Tenant {
        id: eng_id.clone(),
        name: "Engineering".to_string(),
        parent_id: Some(root_id.clone()),
        quotas: TenantQuotas {
            max_users: 200,
            max_roles: 100,
            max_policies: 50,
            max_groups: 20,
            max_access_keys: 400,
            max_sub_tenants: 5,
            api_rate_limit: 500,
        },
        ..root.clone()
    };
    store.create_tenant(eng).await?;
    println!("✅ Created engineering dept");
    
    // Engineering teams
    for team in ["frontend", "backend", "devops"] {
        let team_id = eng_id.child(team);
        let team_tenant = Tenant {
            id: team_id,
            name: team.to_string(),
            parent_id: Some(eng_id.clone()),
            quotas: TenantQuotas {
                max_users: 50,
                max_roles: 20,
                max_policies: 10,
                max_groups: 5,
                max_access_keys: 100,
                max_sub_tenants: 0,
                api_rate_limit: 200,
            },
            ..root.clone()
        };
        store.create_tenant(team_tenant).await?;
        println!("   ✅ Created team: {}", team);
    }
    
    // Query hierarchy
    let all_descendants = store.get_descendants(&root_id).await?;
    println!("\n📊 Total organization size: {} tenants", all_descendants.len() + 1);
    
    let eng_children = store.list_child_tenants(&eng_id).await?;
    println!("📊 Engineering has {} teams", eng_children.len());
    
    Ok(())
}
```

### Example 6: Temporary Sessions (STS)

```rust
use wami::wami::sts::session;
use wami::store::memory::InMemoryStsStore;
use wami::store::traits::SessionStore;
use wami::provider::aws::AwsProvider;
use chrono::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = InMemoryStsStore::new();
    let provider = AwsProvider::new();
    let account = "123456789012";
    
    // Create short-lived session (15 minutes)
    let short_session = session::builder::build_session(
        "short-session".to_string(),
        "AKIASHORT".to_string(),
        "secret-short".to_string(),
        900,  // 15 minutes
        Some("arn:aws:iam::123:role/TempRole".to_string()),
        &provider,
        account
    );
    let created_short = store.create_session(short_session).await?;
    println!("🔑 Short session (15 min):");
    println!("   Token: {}", created_short.session_token);
    println!("   Expires: {}", created_short.expiration);
    
    // Create long-lived session (12 hours)
    let long_session = session::builder::build_session(
        "long-session".to_string(),
        "AKIALONG".to_string(),
        "secret-long".to_string(),
        43200,  // 12 hours
        Some("arn:aws:iam::123:role/AdminRole".to_string()),
        &provider,
        account
    );
    let created_long = store.create_session(long_session).await?;
    println!("\n🔑 Long session (12 hours):");
    println!("   Token: {}", created_long.session_token);
    println!("   Expires: {}", created_long.expiration);
    
    // List all sessions
    let all_sessions = store.list_sessions(None).await?;
    println!("\n📊 Total active sessions: {}", all_sessions.len());
    
    // Clean up
    store.delete_session("short-session").await?;
    println!("\n✅ Deleted short session");
    
    Ok(())
}
```

---

## Real-world Scenarios

### Scenario 1: SaaS Application with Multi-tenant IAM

Build a complete multi-tenant SaaS backend with isolated user management per tenant.

See [Multi-tenant Guide](MULTI_TENANT_GUIDE.md) for the complete implementation.

### Scenario 2: API Service with Temporary Credentials

Create an API service that issues temporary credentials to clients.

See [STS Guide](STS_GUIDE.md) for the complete implementation.

### Scenario 3: Custom SQL Store Backend

Implement a PostgreSQL-backed store for production use.

See [Store Implementation Guide](STORE_IMPLEMENTATION.md) for the complete implementation.

---

## Running the Examples

All examples can be found in the `examples/` directory. Run them with:

```bash
cargo run --example example_name
```

Available examples:
- `arn_architecture_demo.rs` - ARN generation and resource identification

---

## See Also

- **[Getting Started](GETTING_STARTED.md)** - Basic tutorial
- **[API Reference](API_REFERENCE.md)** - Complete API docs
- **[Architecture](ARCHITECTURE.md)** - Design principles
