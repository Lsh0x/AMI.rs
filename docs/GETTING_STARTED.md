# Getting Started with WAMI

Get up and running with WAMI in 10 minutes.

## Table of Contents

- [Installation](#installation)
- [Your First Example](#your-first-example)
- [Understanding the Basics](#understanding-the-basics)
- [Common Patterns](#common-patterns)
- [What's Next](#whats-next)

---

## Installation

### 1. Add WAMI to your project

Add WAMI to your `Cargo.toml`:

```toml
[dependencies]
wami = "0.8.0"
tokio = { version = "1.0", features = ["full"] }
```

### 2. (Optional) Add logging

```toml
[dependencies]
env_logger = "0.11"  # For debug logging
```

### 3. Import in your code

```rust
use wami::wami::identity::user;
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::UserStore;
use wami::provider::aws::AwsProvider;
```

---

## Your First Example

### Step 1: Create a User (2 minutes)

Create a new file `src/main.rs`:

```rust
use wami::wami::identity::user;
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::UserStore;
use wami::provider::aws::AwsProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize storage
    let mut store = InMemoryWamiStore::new();
    
    // 2. Choose a cloud provider
    let provider = AwsProvider::new();
    let account_id = "123456789012";
    
    // 3. Build a user (pure function - no side effects)
    let user = user::builder::build_user(
        "alice".to_string(),
        Some("/engineering/".to_string()),
        &provider,
        account_id
    );
    
    // 4. Store it
    let created = store.create_user(user).await?;
    
    // 5. Success!
    println!("✅ Created: {}", created.arn);
    println!("   User ID: {}", created.user_id);
    println!("   Path: {}", created.path.unwrap_or_default());
    
    Ok(())
}
```

Run it:
```bash
cargo run
```

Output:
```
✅ Created: arn:aws:iam::123456789012:user/engineering/alice
   User ID: AIDACKCEVSQ6C2EXAMPLE
   Path: /engineering/
```

**What just happened?**
1. You initialized an in-memory store
2. You built a `User` domain object using a pure function
3. You stored it in the store
4. The store returned the created user with generated fields (ARN, ID, timestamps)

---

### Step 2: Retrieve and Update (2 minutes)

Add to your `main()`:

```rust
// Get the user back
let retrieved = store.get_user("alice").await?;

if let Some(mut user) = retrieved {
    println!("\n✅ Retrieved: {}", user.user_name);
    
    // Update the path
    user.path = Some("/admin/".to_string());
    let updated = store.update_user(user).await?;
    
    println!("✅ Updated path to: {}", updated.path.unwrap());
}
```

Output:
```
✅ Retrieved: alice
✅ Updated path to: /admin/
```

---

### Step 3: List and Delete (2 minutes)

```rust
// Create more users
for name in ["bob", "charlie"] {
    let user = user::builder::build_user(
        name.to_string(),
        Some("/engineering/".to_string()),
        &provider,
        account_id
    );
    store.create_user(user).await?;
}

// List all users
let (users, _has_more, _marker) = store.list_users(None, None).await?;
println!("\n✅ Total users: {}", users.len());
for u in &users {
    println!("   - {}", u.user_name);
}

// Delete alice
store.delete_user("alice").await?;
println!("\n✅ Deleted: alice");

// Verify deletion
let deleted = store.get_user("alice").await?;
assert!(deleted.is_none());
println!("✅ Confirmed: alice no longer exists");
```

Output:
```
✅ Total users: 3
   - alice
   - bob
   - charlie

✅ Deleted: alice
✅ Confirmed: alice no longer exists
```

---

## Understanding the Basics

### WAMI's Architecture

WAMI separates **domain logic** from **storage**:

```
┌─────────────────────────┐
│   Your Application      │
└──────────┬──────────────┘
           │
     ┌─────┴─────┐
     │           │
┌────▼────┐  ┌──▼────────┐
│ Domain  │  │  Storage  │
│ (wami)  │  │  (store)  │
└─────────┘  └───────────┘
```

**Domain Layer** (`wami::*`):
- Pure functions (no side effects)
- Business logic
- Model definitions

**Storage Layer** (`store::*`):
- Persistence traits
- Implementations (in-memory, SQL, etc.)
- CRUD operations

### Pure Functions vs Store Methods

```rust
// ✅ Pure function - no side effects
let user = user::builder::build_user("alice".into(), None, &provider, account);
// Just creates a User struct, doesn't save anything

// ✅ Store method - persists data
store.create_user(user).await?;
// Saves to storage (memory, database, etc.)
```

**Benefits:**
- Easy to test (pure functions don't need mocks)
- Flexible (use any storage backend)
- Composable (combine functions easily)

---

## Common Patterns

### Pattern 1: Build → Validate → Store

```rust
use wami::wami::identity::user;

// 1. Build
let user = user::builder::build_user("alice".into(), None, &provider, account);

// 2. Validate (optional custom logic)
if user.user_name.len() > 64 {
    return Err("Username too long".into());
}

// 3. Store
store.create_user(user).await?;
```

### Pattern 2: Get → Modify → Update

```rust
// Get existing user
let Some(mut user) = store.get_user("alice").await? else {
    return Err("User not found".into());
};

// Modify
user.path = Some("/new-path/".to_string());

// Update
store.update_user(user).await?;
```

### Pattern 3: List with Pagination

```rust
use wami::types::PaginationParams;

let pagination = PaginationParams {
    max_items: Some(10),
    marker: None,
};

let (users, has_more, next_marker) = store.list_users(
    Some("/engineering/"),  // Path prefix filter
    Some(&pagination)
).await?;

println!("Found {} users", users.len());
if has_more {
    println!("More results available with marker: {:?}", next_marker);
}
```

### Pattern 4: Groups and Memberships

```rust
use wami::wami::identity::group;
use wami::store::traits::GroupStore;

// Create group
let admin_group = group::builder::build_group(
    "admins".to_string(),
    Some("/".to_string()),
    &provider,
    account_id
);
store.create_group(admin_group).await?;

// Add user to group
store.add_user_to_group("admins", "alice").await?;

// List groups for user
let user_groups = store.list_groups_for_user("alice").await?;
println!("Alice is in {} groups", user_groups.len());

// Remove from group
store.remove_user_from_group("admins", "alice").await?;
```

### Pattern 5: Roles and Trust Policies

```rust
use wami::wami::identity::role;
use wami::store::traits::RoleStore;

// Define trust policy
let trust_policy = r#"{
    "Version": "2012-10-17",
    "Statement": [{
        "Effect": "Allow",
        "Principal": {"Service": "lambda.amazonaws.com"},
        "Action": "sts:AssumeRole"
    }]
}"#;

// Create role
let lambda_role = role::builder::build_role(
    "LambdaExecutionRole".to_string(),
    trust_policy.to_string(),
    Some("/service/".to_string()),
    Some("Role for Lambda functions".to_string()),
    None,  // tags
    &provider,
    account_id
);

let created_role = store.create_role(lambda_role).await?;
println!("Created role: {}", created_role.arn);
```

### Pattern 6: Access Keys

```rust
use wami::wami::credentials::access_key;
use wami::store::traits::AccessKeyStore;

// Create access key for user
let key = access_key::builder::build_access_key(
    "alice".to_string(),
    &provider,
    account_id
);

let created_key = store.create_access_key(key).await?;

println!("Access Key ID: {}", created_key.access_key_id);
println!("Secret: {}", created_key.secret_access_key);
println!("Status: {:?}", created_key.status);

// List all keys for user
let keys = store.list_access_keys("alice").await?;
println!("Alice has {} access keys", keys.len());
```

### Pattern 7: Temporary Sessions (STS)

```rust
use wami::wami::sts::session;
use wami::store::traits::SessionStore;
use wami::store::memory::InMemoryStsStore;

let mut sts_store = InMemoryStsStore::new();

// Create temporary session
let session = session::builder::build_session(
    "session-abc123".to_string(),
    "AKIAIOSFODNN7EXAMPLE".to_string(),
    "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
    3600,  // 1 hour
    Some("arn:aws:iam::123456789012:role/AdminRole".to_string()),
    &provider,
    account_id
);

let created_session = sts_store.create_session(session).await?;

println!("Session token: {}", created_session.session_token);
println!("Expires at: {}", created_session.expiration);

// Check if session exists
let retrieved = sts_store.get_session("session-abc123").await?;
assert!(retrieved.is_some());
```

### Pattern 8: Multi-tenant Setup

```rust
use wami::wami::tenant::{Tenant, TenantId, TenantQuotas, TenantStatus, TenantType, QuotaMode};
use wami::store::memory::InMemoryTenantStore;
use wami::store::traits::TenantStore;

let mut tenant_store = InMemoryTenantStore::new();

// Create root tenant
let root_id = TenantId::root("acme-corp");
let root_tenant = Tenant {
    id: root_id.clone(),
    name: "Acme Corporation".to_string(),
    parent_id: None,
    organization: Some("Acme Inc.".to_string()),
    tenant_type: TenantType::Root,
    provider_accounts: std::collections::HashMap::new(),
    arn: "arn:wami:tenant::acme-corp".to_string(),
    providers: vec![],
    created_at: chrono::Utc::now(),
    status: TenantStatus::Active,
    quotas: TenantQuotas::default(),
    quota_mode: QuotaMode::Override,
    max_child_depth: 5,
    can_create_sub_tenants: true,
    admin_principals: vec![],
    metadata: std::collections::HashMap::new(),
    billing_info: None,
};

tenant_store.create_tenant(root_tenant).await?;

// Create child tenant
let eng_id = root_id.child("engineering");
let eng_tenant = Tenant {
    id: eng_id.clone(),
    name: "Engineering Team".to_string(),
    parent_id: Some(root_id.clone()),
    // ... same fields as above
};

tenant_store.create_tenant(eng_tenant).await?;

// Query hierarchy
let children = tenant_store.list_child_tenants(&root_id).await?;
println!("Root tenant has {} children", children.len());
```

---

## Error Handling

WAMI uses `Result<T, AmiError>` for all operations:

```rust
use wami::error::AmiError;

match store.get_user("nonexistent").await {
    Ok(Some(user)) => println!("Found: {}", user.user_name),
    Ok(None) => println!("User not found"),
    Err(AmiError::ValidationError(msg)) => println!("Validation failed: {}", msg),
    Err(AmiError::ResourceNotFound { resource }) => println!("Not found: {}", resource),
    Err(e) => println!("Error: {}", e),
}
```

Common error types:
- `ValidationError` - Invalid input
- `ResourceNotFound` - Resource doesn't exist
- `ResourceExists` - Duplicate resource
- `AccessDenied` - Permission denied
- `InternalError` - Storage or system error

---

## What's Next?

### 📚 Read More Guides

- **[Architecture](ARCHITECTURE.md)** - Understand WAMI's design
- **[API Reference](API_REFERENCE.md)** - Complete API documentation
- **[IAM Guide](IAM_GUIDE.md)** - Deep dive into IAM operations
- **[STS Guide](STS_GUIDE.md)** - Temporary credentials and sessions
- **[Multi-tenant Guide](MULTI_TENANT_GUIDE.md)** - Tenant isolation

### 🔨 Build Something

Try implementing:
- [ ] User management API
- [ ] Role-based access control system
- [ ] Multi-tenant SaaS backend
- [ ] Temporary credentials service
- [ ] Custom storage backend (SQL, Redis)

### 🎯 Explore Examples

Check out the [examples directory](../examples/) for complete working examples.

### 💬 Get Help

- **Issues**: [GitHub Issues](https://github.com/lsh0x/wami/issues)
- **Discussions**: [GitHub Discussions](https://github.com/lsh0x/wami/discussions)

---

**Happy coding! 🦀**
