# Getting Started with WAMI

Get up and running in 5 minutes.

## Table of Contents

- [Installation](#installation)
- [Your First Example](#your-first-example)
- [Understanding the Basics](#understanding-the-basics)
- [Common Patterns](#common-patterns)
- [Next Steps](#next-steps)

## Installation

Add WAMI to your `Cargo.toml`:

```toml
[dependencies]
wami = "0.8.0"
tokio = { version = "1.0", features = ["full"] }
env_logger = "0.11"  # Optional: for logging
```

## Your First Example

### 1️⃣ Create a User (30 seconds)

```rust
use wami::{MemoryIamClient, CreateUserRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize store and client
    let store = wami::create_memory_store();
    let mut iam = MemoryIamClient::new(store);
    
    // Create a user
    let user = iam.create_user(CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/engineering/".to_string()),
        permissions_boundary: None,
        tags: None,
    }).await?;
    
    println!("Created: {}", user.data.unwrap().arn);
    // Output: arn:aws:iam::123456789012:user/engineering/alice
    
    Ok(())
}
```

Run it:
```bash
cargo run
```

### 2️⃣ Complete Workflow (2 minutes)

```rust
use wami::{
    MemoryIamClient, MemoryStsClient,
    CreateUserRequest, CreateAccessKeyRequest, AssumeRoleRequest, CreateRoleRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Enable logging
    
    // Setup
    let store = wami::create_memory_store();
    let mut iam = MemoryIamClient::new(store.clone());
    let mut sts = MemoryStsClient::new(store);
    
    // 1. Create user
    let user = iam.create_user(CreateUserRequest {
        user_name: "developer".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    }).await?;
    println!("✓ Created user: {}", user.data.as_ref().unwrap().user_name);
    
    // 2. Create access keys
    let keys = iam.create_access_key(CreateAccessKeyRequest {
        user_name: "developer".to_string(),
    }).await?;
    let key = keys.data.unwrap();
    println!("✓ Access Key: {}", key.access_key_id);
    println!("✓ Secret Key: {}", key.secret_access_key.unwrap());
    
    // 3. Create role
    let role = iam.create_role(CreateRoleRequest {
        role_name: "DataScientist".to_string(),
        assume_role_policy_document: r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {"Service": "ec2.amazonaws.com"},
                "Action": "sts:AssumeRole"
            }]
        }"#.to_string(),
        path: None,
        description: Some("Data science role".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: None,
    }).await?;
    println!("✓ Created role: {}", role.data.as_ref().unwrap().role_name);
    
    // 4. Get temporary credentials
    let creds = sts.assume_role(AssumeRoleRequest {
        role_arn: role.data.unwrap().arn,
        role_session_name: "session1".to_string(),
        duration_seconds: Some(3600),
        external_id: None,
        policy: None,
    }).await?;
    println!("✓ Temporary credentials:");
    println!("  Access Key: {}", creds.data.as_ref().unwrap().credentials.access_key_id);
    
    // 5. Get caller identity
    let identity = sts.get_caller_identity().await?;
    println!("✓ Caller: {}", identity.data.unwrap().arn);
    
    Ok(())
}
```

### 3️⃣ Multi-Cloud Support (5 minutes)

```rust
use wami::{MemoryIamClient, CreateUserRequest, InMemoryStore};
use wami::provider::{AwsProvider, GcpProvider, AzureProvider};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // AWS Provider (default)
    let aws_store = InMemoryStore::with_provider(Arc::new(AwsProvider::new()));
    let mut aws_iam = MemoryIamClient::new(aws_store);
    
    // GCP Provider
    let gcp_store = InMemoryStore::with_provider(Arc::new(GcpProvider::new("my-project")));
    let mut gcp_iam = MemoryIamClient::new(gcp_store);
    
    // Azure Provider
    let azure_store = InMemoryStore::with_provider(
        Arc::new(AzureProvider::new("subscription-id", "resource-group"))
    );
    let mut azure_iam = MemoryIamClient::new(azure_store);
    
    let request = CreateUserRequest {
        user_name: "alice".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    };
    
    // Each provider generates cloud-specific identifiers
    let aws_user = aws_iam.create_user(request.clone()).await?;
    println!("AWS: {}", aws_user.data.as_ref().unwrap().arn);
    // → arn:aws:iam::123456789012:user/alice
    
    let gcp_user = gcp_iam.create_user(request.clone()).await?;
    println!("GCP: {}", gcp_user.data.as_ref().unwrap().arn);
    // → projects/my-project/serviceAccounts/alice@my-project.iam.gserviceaccount.com
    
    let azure_user = azure_iam.create_user(request).await?;
    println!("Azure: {}", azure_user.data.unwrap().arn);
    // → /subscriptions/subscription-id/.../users/alice
    
    Ok(())
}
```

## Understanding the Basics

### Core Concepts

```
┌─────────────┐
│   Client    │ ← Your code interacts with clients
└──────┬──────┘
       │
┌──────▼──────┐
│    Store    │ ← Manages persistence (memory, database, etc.)
└──────┬──────┘
       │
┌──────▼──────┐
│  Provider   │ ← Generates cloud-specific IDs and ARNs
└─────────────┘
```

### 1. **Clients** - Your API Interface

```rust
let mut iam = MemoryIamClient::new(store);      // IAM operations
let mut sts = MemoryStsClient::new(store);      // STS operations
let mut sso = MemorySsoAdminClient::new(store); // SSO operations
let mut tenant = TenantClient::new(store, principal); // Tenant operations
```

### 2. **Store** - Data Persistence

```rust
// In-memory (data lost on restart)
let store = wami::create_memory_store();

// Custom account ID
let store = wami::create_memory_store_with_account_id("123456789012".to_string());

// Custom provider
use std::sync::Arc;
let provider = Arc::new(AwsProvider::new());
let store = InMemoryStore::with_provider(provider);

// Your own implementation (database, cloud, etc.)
let store = MyDatabaseStore::new(connection);
```

### 3. **Providers** - Cloud Abstraction

```rust
// AWS (default)
Arc::new(AwsProvider::new())

// GCP
Arc::new(GcpProvider::new("project-id"))

// Azure
Arc::new(AzureProvider::new("subscription-id", "resource-group"))

// Custom
Arc::new(CustomProvider::builder()
    .name("mycloud")
    .arn_template("mycloud://{account}/{type}/{name}")
    .build())
```

## Common Patterns

### Pattern 1: Get Account ID

```rust
let store = wami::create_memory_store();
let account_id = wami::get_account_id_from_store(&store);
println!("Account: {}", account_id);
```

### Pattern 2: Enable Logging

```rust
env_logger::init();
// Shows: account ID generation, operations, ARNs, etc.
```

### Pattern 3: Error Handling

```rust
use wami::AmiError;

match iam.create_user(request).await {
    Ok(response) => {
        println!("Success: {:?}", response.data);
    }
    Err(AmiError::ResourceExists { resource }) => {
        println!("Already exists: {}", resource);
    }
    Err(AmiError::InvalidParameter { message }) => {
        println!("Invalid: {}", message);
    }
    Err(AmiError::ResourceNotFound { resource }) => {
        println!("Not found: {}", resource);
    }
    Err(e) => {
        println!("Error: {:?}", e);
    }
}
```

### Pattern 4: Reuse Store Across Clients

```rust
let store = wami::create_memory_store();

// Share the same store
let mut iam = MemoryIamClient::new(store.clone());
let mut sts = MemoryStsClient::new(store.clone());
let mut sso = MemorySsoAdminClient::new(store);

// All clients see the same data
```

### Pattern 5: Tags and Metadata

```rust
use wami::Tag;

let user = iam.create_user(CreateUserRequest {
    user_name: "alice".to_string(),
    path: Some("/engineering/".to_string()),
    permissions_boundary: None,
    tags: Some(vec![
        Tag {
            key: "Environment".to_string(),
            value: "Production".to_string(),
        },
        Tag {
            key: "Team".to_string(),
            value: "Engineering".to_string(),
        },
    ]),
}).await?;
```

## Next Steps

### Learn More

- **[IAM Operations](IAM_GUIDE.md)** - Complete guide to users, roles, policies
- **[STS Operations](STS_GUIDE.md)** - Temporary credentials and sessions
- **[Multi-Tenant](MULTI_TENANT_GUIDE.md)** - Tenant isolation for SaaS
- **[Multicloud](MULTICLOUD_PROVIDERS.md)** - AWS, GCP, Azure providers
- **[Store Implementation](STORE_IMPLEMENTATION.md)** - Add database persistence

### Try Examples

```bash
# Browse examples
ls examples/

# Run an example
cargo run --example multi_tenant
```

See [Examples Catalog](EXAMPLES.md) for all available examples.

### Read API Docs

```bash
cargo doc --open
```

Or visit [docs.rs/wami](https://docs.rs/wami)

## Troubleshooting

### Issue: Account ID changes each run

**Solution**: Use a fixed account ID

```rust
let store = wami::create_memory_store_with_account_id("123456789012".to_string());
```

### Issue: Can't see log output

**Solution**: Enable logging

```rust
env_logger::init(); // Add at start of main()
```

### Issue: Data doesn't persist

**Solution**: In-memory store is ephemeral. Implement a custom store for persistence.

See [Store Implementation Guide](STORE_IMPLEMENTATION.md)

## Help & Support

- 📖 [Full Documentation](README.md)
- 🐛 [Report Issues](https://github.com/lsh0x/wami/issues)
- 💬 [Discussions](https://github.com/lsh0x/wami/discussions)

