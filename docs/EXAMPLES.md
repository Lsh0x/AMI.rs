# Examples Catalog

Comprehensive collection of WAMI code examples.

## Quick Links

- [Basic Examples](#basic-examples)
- [IAM Examples](#iam-examples)
- [STS Examples](#sts-examples)
- [Multi-Tenant Examples](#multi-tenant-examples)
- [Multi-Cloud Examples](#multi-cloud-examples)
- [Testing Examples](#testing-examples)

## Running Examples

```bash
# List all examples
ls examples/

# Run an example
cargo run --example <example_name>

# Example:
cargo run --example multi_tenant
```

## Basic Examples

### Hello WAMI

**File**: `examples/hello_wami.rs`

```rust
use wami::{MemoryIamClient, CreateUserRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = wami::create_memory_store();
    let mut iam = MemoryIamClient::new(store);
    
    let user = iam.create_user(CreateUserRequest {
        user_name: "alice".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    }).await?;
    
    println!("Created: {}", user.data.unwrap().arn);
    Ok(())
}
```

**Run**: `cargo run --example hello_wami`

### Complete Workflow

**File**: `examples/complete_workflow.rs`

Demonstrates:
- Creating users
- Generating access keys
- Creating roles
- Getting temporary credentials
- Checking caller identity

**Run**: `cargo run --example complete_workflow`

## IAM Examples

### User Management

**File**: `examples/user_management.rs`

```rust
use wami::{MemoryIamClient, CreateUserRequest, Tag};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = wami::create_memory_store();
    let mut iam = MemoryIamClient::new(store);
    
    // Create user with tags
    let user = iam.create_user(CreateUserRequest {
        user_name: "developer".to_string(),
        path: Some("/engineering/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            Tag {
                key: "Department".to_string(),
                value: "Engineering".to_string(),
            },
            Tag {
                key: "Team".to_string(),
                value: "Backend".to_string(),
            },
        ]),
    }).await?;
    
    println!("✓ Created: {}", user.data.as_ref().unwrap().user_name);
    
    // List all users
    let users = iam.list_users(None, None).await?;
    println!("\n📋 All users:");
    for u in users.data.unwrap().users {
        println!("  - {} ({})", u.user_name, u.path);
    }
    
    // Update user
    use wami::UpdateUserRequest;
    iam.update_user(UpdateUserRequest {
        user_name: "developer".to_string(),
        new_path: Some("/senior/".to_string()),
        new_user_name: None,
    }).await?;
    
    println!("\n✓ Updated user path");
    
    // Get updated user
    let updated = iam.get_user("developer").await?;
    println!("  New path: {}", updated.data.unwrap().path);
    
    Ok(())
}
```

**Run**: `cargo run --example user_management`

### Policy Management

**File**: `examples/policy_management.rs`

Demonstrates:
- Creating managed policies
- Attaching policies to users/roles
- Inline policies
- Permissions boundaries

**Run**: `cargo run --example policy_management`

### Access Keys Rotation

**File**: `examples/access_key_rotation.rs`

```rust
use wami::{MemoryIamClient, CreateAccessKeyRequest, UpdateAccessKeyRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = wami::create_memory_store();
    let mut iam = MemoryIamClient::new(store);
    
    // Setup: Create user
    use wami::CreateUserRequest;
    iam.create_user(CreateUserRequest {
        user_name: "app-service".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    }).await?;
    
    // Create initial access key
    let key1 = iam.create_access_key(CreateAccessKeyRequest {
        user_name: "app-service".to_string(),
    }).await?;
    let old_key_id = key1.data.as_ref().unwrap().access_key_id.clone();
    println!("✓ Created initial key: {}", old_key_id);
    
    // Rotation process:
    
    // 1. Create new key
    let key2 = iam.create_access_key(CreateAccessKeyRequest {
        user_name: "app-service".to_string(),
    }).await?;
    let new_key_id = key2.data.as_ref().unwrap().access_key_id.clone();
    println!("✓ Created new key: {}", new_key_id);
    
    // 2. Update application configuration with new key
    println!("⏳ Deploying new key to application...");
    
    // 3. Deactivate old key
    iam.update_access_key(UpdateAccessKeyRequest {
        user_name: "app-service".to_string(),
        access_key_id: old_key_id.clone(),
        status: "Inactive".to_string(),
    }).await?;
    println!("✓ Deactivated old key");
    
    // 4. Monitor for errors...
    println!("⏳ Monitoring application...");
    
    // 5. Delete old key after verification period
    iam.delete_access_key("app-service", &old_key_id).await?;
    println!("✓ Deleted old key");
    
    println!("\n✅ Key rotation complete!");
    
    Ok(())
}
```

**Run**: `cargo run --example access_key_rotation`

## STS Examples

### Assume Role

**File**: `examples/assume_role.rs`

```rust
use wami::{MemoryIamClient, MemoryStsClient, CreateRoleRequest};
use wami::sts::assume_role::requests::AssumeRoleRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = wami::create_memory_store();
    let mut iam = MemoryIamClient::new(store.clone());
    let mut sts = MemoryStsClient::new(store);
    
    // Create a role
    let role = iam.create_role(CreateRoleRequest {
        role_name: "DataAccess".to_string(),
        assume_role_policy_document: r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {"Service": "ec2.amazonaws.com"},
                "Action": "sts:AssumeRole"
            }]
        }"#.to_string(),
        path: None,
        description: Some("Data access role".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: None,
    }).await?;
    
    let role_arn = role.data.unwrap().arn;
    
    // Assume the role
    let credentials = sts.assume_role(AssumeRoleRequest {
        role_arn: role_arn.clone(),
        role_session_name: "my-session".to_string(),
        duration_seconds: Some(3600),
        external_id: None,
        policy: None,
    }).await?;
    
    let creds = credentials.data.unwrap();
    println!("✓ Assumed role: {}", role_arn);
    println!("  Access Key: {}", creds.credentials.access_key_id);
    println!("  Session Token: {}", creds.credentials.session_token.unwrap());
    println!("  Expires: {}", creds.credentials.expiration);
    
    Ok(())
}
```

**Run**: `cargo run --example assume_role`

### Session Token

**File**: `examples/session_token.rs`

Demonstrates:
- Getting session tokens
- Temporary credentials with MFA
- Session duration control

**Run**: `cargo run --example session_token`

## Multi-Tenant Examples

### Hierarchical Tenants

**File**: `examples/multi_tenant.rs`

```rust
use wami::tenant::{TenantClient, Tenant, TenantType, TenantQuotas};
use wami::tenant::store::CreateTenantRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = wami::create_memory_store();
    let principal = "admin-user".to_string();
    let mut tenant_client = TenantClient::new(store, principal);
    
    // Create root tenant (organization)
    let root = tenant_client.create_root_tenant(CreateTenantRequest {
        name: "ACME Corp".to_string(),
        tenant_type: TenantType::Organization,
        quotas: Some(TenantQuotas {
            max_users: Some(1000),
            max_roles: Some(500),
            max_policies: Some(200),
            max_groups: Some(100),
        }),
        metadata: None,
        provider_accounts: None,
    }).await?;
    
    println!("✓ Root tenant: {} ({})", root.name, root.id);
    
    // Create department (sub-tenant)
    let engineering = tenant_client.create_sub_tenant(
        &root.id,
        CreateTenantRequest {
            name: "Engineering".to_string(),
            tenant_type: TenantType::Department,
            quotas: Some(TenantQuotas {
                max_users: Some(200),
                max_roles: Some(50),
                max_policies: Some(30),
                max_groups: Some(20),
            }),
            metadata: None,
            provider_accounts: None,
        },
    ).await?;
    
    println!("✓ Sub-tenant: {} ({})", engineering.name, engineering.id);
    
    // Create team (sub-sub-tenant)
    let backend_team = tenant_client.create_sub_tenant(
        &engineering.id,
        CreateTenantRequest {
            name: "Backend Team".to_string(),
            tenant_type: TenantType::Team,
            quotas: Some(TenantQuotas {
                max_users: Some(50),
                max_roles: Some(10),
                max_policies: Some(5),
                max_groups: Some(5),
            }),
            metadata: None,
            provider_accounts: None,
        },
    ).await?;
    
    println!("✓ Team: {} ({})", backend_team.name, backend_team.id);
    
    // List children
    let children = tenant_client.list_child_tenants(&engineering.id).await?;
    println!("\n📋 Engineering sub-tenants:");
    for child in children {
        println!("  - {} ({})", child.name, child.id);
    }
    
    Ok(())
}
```

**Run**: `cargo run --example multi_tenant`

### Tenant Isolation

**File**: `examples/tenant_isolation.rs`

Demonstrates:
- Creating tenant-scoped resources
- Tenant-aware ARNs
- Permission checks across tenants

**Run**: `cargo run --example tenant_isolation`

## Multi-Cloud Examples

### AWS Provider

**File**: `examples/aws_provider.rs`

```rust
use wami::{InMemoryStore, MemoryIamClient, CreateUserRequest};
use wami::provider::AwsProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(AwsProvider::new());
    let store = InMemoryStore::with_provider(provider);
    let mut iam = MemoryIamClient::new(store);
    
    let user = iam.create_user(CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/team/".to_string()),
        permissions_boundary: None,
        tags: None,
    }).await?;
    
    println!("AWS ARN: {}", user.data.unwrap().arn);
    // Output: arn:aws:iam::123456789012:user/team/alice
    
    Ok(())
}
```

**Run**: `cargo run --example aws_provider`

### GCP Provider

**File**: `examples/gcp_provider.rs`

```rust
use wami::{InMemoryStore, MemoryIamClient, CreateUserRequest};
use wami::provider::GcpProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(GcpProvider::new("my-project"));
    let store = InMemoryStore::with_provider(provider);
    let mut iam = MemoryIamClient::new(store);
    
    let user = iam.create_user(CreateUserRequest {
        user_name: "alice".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    }).await?;
    
    println!("GCP ID: {}", user.data.unwrap().arn);
    // Output: projects/my-project/serviceAccounts/alice@my-project.iam.gserviceaccount.com
    
    Ok(())
}
```

**Run**: `cargo run --example gcp_provider`

### Multi-Cloud Abstraction

**File**: `examples/multicloud.rs`

Demonstrates using AWS, GCP, and Azure providers simultaneously.

**Run**: `cargo run --example multicloud`

## Testing Examples

### Policy Validation

**File**: `examples/policy_validation.rs`

```rust
use wami::iam::{PolicyEvaluator, PolicyDocument};
use wami::iam::policy_evaluation::Action;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let policy_json = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": ["s3:GetObject", "s3:PutObject"],
            "Resource": "arn:aws:s3:::my-bucket/*"
        }]
    }"#;
    
    let policy: PolicyDocument = serde_json::from_str(policy_json)?;
    let evaluator = PolicyEvaluator::new();
    
    // Test cases
    let test_cases = vec![
        ("s3:GetObject", "arn:aws:s3:::my-bucket/file.txt", true),
        ("s3:PutObject", "arn:aws:s3:::my-bucket/file.txt", true),
        ("s3:DeleteObject", "arn:aws:s3:::my-bucket/file.txt", false),
        ("s3:GetObject", "arn:aws:s3:::other-bucket/file.txt", false),
    ];
    
    println!("🧪 Policy Validation Tests\n");
    
    for (action, resource, expected) in test_cases {
        let result = evaluator.evaluate(
            &[policy.clone()],
            &Action::from(action),
            resource,
            &Default::default(),
        )?;
        
        let status = if result == expected { "✓" } else { "✗" };
        println!("{} {} on {} → {}", status, action, resource, result);
    }
    
    Ok(())
}
```

**Run**: `cargo run --example policy_validation`

### Integration Test

**File**: `tests/integration_test.rs`

Full integration test demonstrating:
- IAM operations
- STS operations
- SSO Admin operations
- Error handling

**Run**: `cargo test --test integration_test`

## Advanced Examples

### Custom Store

**File**: `examples/custom_store.rs`

Demonstrates implementing a custom storage backend.

**Run**: `cargo run --example custom_store`

### Database Store (PostgreSQL)

**File**: `examples/postgres_store/`

Complete PostgreSQL implementation with:
- Schema migrations
- Connection pooling
- Transaction support
- Caching layer

**Run**: 
```bash
cd examples/postgres_store
cargo run
```

### Redis Session Store

**File**: `examples/redis_store.rs`

Using Redis for STS sessions with automatic expiration.

**Run**: `cargo run --example redis_store`

## Testing Your Code

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wami::{MemoryIamClient, CreateUserRequest};
    
    #[tokio::test]
    async fn test_create_user() {
        let store = wami::create_memory_store();
        let mut iam = MemoryIamClient::new(store);
        
        let result = iam.create_user(CreateUserRequest {
            user_name: "test-user".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        }).await;
        
        assert!(result.is_ok());
        let user = result.unwrap().data.unwrap();
        assert_eq!(user.user_name, "test-user");
    }
}
```

### Integration Tests

```rust
// tests/my_integration_test.rs
use wami::*;

#[tokio::test]
async fn test_complete_workflow() {
    let store = create_memory_store();
    let mut iam = MemoryIamClient::new(store.clone());
    let mut sts = MemoryStsClient::new(store);
    
    // Create user
    let user = iam.create_user(CreateUserRequest {
        user_name: "test".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    }).await.unwrap();
    
    // Create access key
    let keys = iam.create_access_key(CreateAccessKeyRequest {
        user_name: "test".to_string(),
    }).await.unwrap();
    
    // Verify
    assert!(keys.data.is_some());
}
```

## Next Steps

- **[Getting Started](GETTING_STARTED.md)** - 5-minute quickstart
- **[IAM Guide](IAM_GUIDE.md)** - Complete IAM operations
- **[STS Guide](STS_GUIDE.md)** - Temporary credentials
- **[Multi-Tenant](MULTI_TENANT_GUIDE.md)** - Tenant isolation
- **[Multicloud](MULTICLOUD_PROVIDERS.md)** - Cloud providers

## Contributing Examples

Have a useful example? Submit a PR!

1. Create your example in `examples/`
2. Add documentation here
3. Test it: `cargo run --example your_example`
4. Submit PR

## Support

Questions? Open an issue on [GitHub](https://github.com/lsh0x/wami/issues).

