# IAM Operations Guide

Complete guide to Identity and Access Management with WAMI.

## Overview

WAMI provides AWS-compatible IAM operations for managing:
- **Users** - Individual identities
- **Roles** - Assumable identities for services and applications
- **Policies** - Permission documents
- **Groups** - Collections of users
- **Access Keys** - Programmatic access credentials
- **MFA Devices** - Multi-factor authentication
- **And more...**

## Quick Reference

| Operation | Description | Example |
|-----------|-------------|---------|
| `create_user` | Create a new user | `iam.create_user(req).await?` |
| `get_user` | Fetch user details | `iam.get_user("alice").await?` |
| `list_users` | List all users | `iam.list_users(None, None).await?` |
| `delete_user` | Remove a user | `iam.delete_user("alice").await?` |
| `create_role` | Create a role | `iam.create_role(req).await?` |
| `create_policy` | Create a managed policy | `iam.create_policy(req).await?` |
| `attach_user_policy` | Attach policy to user | `iam.attach_user_policy("alice", "arn").await?` |
| `create_access_key` | Generate access keys | `iam.create_access_key(req).await?` |

## Users

### Create User

```rust
use wami::{MemoryIamClient, CreateUserRequest, Tag};

let mut iam = MemoryIamClient::new(store);

let user = iam.create_user(CreateUserRequest {
    user_name: "alice".to_string(),
    path: Some("/engineering/".to_string()),
    permissions_boundary: None,
    tags: Some(vec![
        Tag {
            key: "Department".to_string(),
            value: "Engineering".to_string(),
        },
    ]),
}).await?;

println!("Created: {}", user.data.unwrap().arn);
// Output: arn:aws:iam::123456789012:user/engineering/alice
```

### Get User

```rust
let user = iam.get_user("alice").await?;
if let Some(u) = user.data {
    println!("User: {} (created: {})", u.user_name, u.create_date);
}
```

### List Users

```rust
let response = iam.list_users(
    Some("/engineering/"), // Path prefix
    None,                  // No pagination marker
).await?;

for user in response.data.unwrap().users {
    println!("- {} ({})", user.user_name, user.arn);
}
```

### Update User

```rust
use wami::UpdateUserRequest;

iam.update_user(UpdateUserRequest {
    user_name: "alice".to_string(),
    new_path: Some("/admin/".to_string()),
    new_user_name: None,
}).await?;
```

### Delete User

```rust
iam.delete_user("alice").await?;
```

## Roles

### Create Role

```rust
use wami::CreateRoleRequest;

let role = iam.create_role(CreateRoleRequest {
    role_name: "DataScientist".to_string(),
    assume_role_policy_document: r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": {
                "Service": "ec2.amazonaws.com"
            },
            "Action": "sts:AssumeRole"
        }]
    }"#.to_string(),
    path: Some("/roles/".to_string()),
    description: Some("Role for data science workloads".to_string()),
    max_session_duration: Some(3600),
    permissions_boundary: None,
    tags: None,
}).await?;

println!("Role ARN: {}", role.data.unwrap().arn);
```

### Get Role

```rust
let role = iam.get_role("DataScientist").await?;
```

### List Roles

```rust
let roles = iam.list_roles(Some("/roles/"), None).await?;
```

### Attach Role Policy

```rust
iam.attach_role_policy("DataScientist", "arn:aws:iam::aws:policy/ReadOnlyAccess").await?;
```

### Delete Role

```rust
// Must detach all policies first
iam.detach_role_policy("DataScientist", "arn:aws:iam::aws:policy/ReadOnlyAccess").await?;
iam.delete_role("DataScientist").await?;
```

## Policies

### Create Managed Policy

```rust
use wami::CreatePolicyRequest;

let policy = iam.create_policy(CreatePolicyRequest {
    policy_name: "S3ReadOnly".to_string(),
    policy_document: r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": [
                "s3:GetObject",
                "s3:ListBucket"
            ],
            "Resource": [
                "arn:aws:s3:::my-bucket",
                "arn:aws:s3:::my-bucket/*"
            ]
        }]
    }"#.to_string(),
    path: Some("/policies/".to_string()),
    description: Some("Read-only access to S3".to_string()),
    tags: None,
}).await?;

let policy_arn = policy.data.unwrap().arn;
```

### Get Policy

```rust
let policy = iam.get_policy(&policy_arn).await?;
```

### List Policies

```rust
let policies = iam.list_policies(
    None,  // Scope: All, AWS, Local
    false, // Only attached
    Some("/policies/"),
    None,  // No pagination
).await?;
```

### Delete Policy

```rust
iam.delete_policy(&policy_arn).await?;
```

## Groups

### Create Group

```rust
use wami::CreateGroupRequest;

let group = iam.create_group(CreateGroupRequest {
    group_name: "Developers".to_string(),
    path: Some("/teams/".to_string()),
}).await?;
```

### Add User to Group

```rust
iam.add_user_to_group("alice", "Developers").await?;
```

### List Groups for User

```rust
let groups = iam.list_groups_for_user("alice", None, None).await?;
```

### Attach Group Policy

```rust
iam.attach_group_policy("Developers", policy_arn).await?;
```

### Remove User from Group

```rust
iam.remove_user_from_group("alice", "Developers").await?;
```

## Access Keys

### Create Access Key

```rust
use wami::CreateAccessKeyRequest;

let key_response = iam.create_access_key(CreateAccessKeyRequest {
    user_name: "alice".to_string(),
}).await?;

let key = key_response.data.unwrap();
println!("Access Key ID: {}", key.access_key_id);
println!("Secret Key: {}", key.secret_access_key.unwrap());
// ⚠️ Secret is only returned once! Store it securely.
```

### List Access Keys

```rust
let keys = iam.list_access_keys("alice", None, None).await?;
```

### Update Access Key Status

```rust
use wami::UpdateAccessKeyRequest;

iam.update_access_key(UpdateAccessKeyRequest {
    user_name: "alice".to_string(),
    access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
    status: "Inactive".to_string(), // or "Active"
}).await?;
```

### Delete Access Key

```rust
iam.delete_access_key("alice", "AKIAIOSFODNN7EXAMPLE").await?;
```

## Login Profiles (Passwords)

### Create Login Profile

```rust
use wami::CreateLoginProfileRequest;

iam.create_login_profile(CreateLoginProfileRequest {
    user_name: "alice".to_string(),
    password: "SecurePassword123!".to_string(),
    password_reset_required: Some(true),
}).await?;
```

### Update Password

```rust
use wami::UpdateLoginProfileRequest;

iam.update_login_profile(UpdateLoginProfileRequest {
    user_name: "alice".to_string(),
    password: Some("NewSecurePassword456!".to_string()),
    password_reset_required: Some(false),
}).await?;
```

### Delete Login Profile

```rust
iam.delete_login_profile("alice").await?;
```

## MFA Devices

### Enable Virtual MFA

```rust
use wami::{EnableMfaDeviceRequest, CreateVirtualMfaDeviceRequest};

// 1. Create virtual MFA device
let device = iam.create_virtual_mfa_device(CreateVirtualMfaDeviceRequest {
    virtual_mfa_device_name: "alice-mfa".to_string(),
    path: None,
    tags: None,
}).await?;

// 2. Enable it for user
iam.enable_mfa_device(EnableMfaDeviceRequest {
    user_name: "alice".to_string(),
    serial_number: device.data.unwrap().serial_number,
    authentication_code_1: "123456".to_string(),
    authentication_code_2: "234567".to_string(),
}).await?;
```

### List MFA Devices

```rust
let devices = iam.list_mfa_devices("alice", None, None).await?;
```

### Deactivate MFA

```rust
iam.deactivate_mfa_device("alice", "arn:aws:iam::123456789012:mfa/alice-mfa").await?;
```

## Advanced Operations

### Attach Inline Policy

```rust
use wami::PutUserPolicyRequest;

iam.put_user_policy(PutUserPolicyRequest {
    user_name: "alice".to_string(),
    policy_name: "inline-s3-access".to_string(),
    policy_document: r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": "s3:*",
            "Resource": "arn:aws:s3:::alice-bucket/*"
        }]
    }"#.to_string(),
}).await?;
```

### Set Permissions Boundary

```rust
use wami::PutUserPermissionsBoundaryRequest;

iam.put_user_permissions_boundary(PutUserPermissionsBoundaryRequest {
    user_name: "alice".to_string(),
    permissions_boundary: "arn:aws:iam::123456789012:policy/DeveloperBoundary".to_string(),
}).await?;
```

### Tag Resources

```rust
use wami::TagUserRequest;

iam.tag_user(TagUserRequest {
    user_name: "alice".to_string(),
    tags: vec![
        Tag {
            key: "CostCenter".to_string(),
            value: "Engineering".to_string(),
        },
    ],
}).await?;
```

## Best Practices

### 1. Use Paths for Organization

```rust
// Organize by team/department
"/teams/engineering/"
"/teams/marketing/"

// Organize by environment
"/production/"
"/staging/"
"/development/"
```

### 2. Always Tag Resources

```rust
tags: Some(vec![
    Tag { key: "Environment".into(), value: "Production".into() },
    Tag { key: "Owner".into(), value: "alice@example.com".into() },
    Tag { key: "CostCenter".into(), value: "Engineering".into() },
])
```

### 3. Use Permissions Boundaries

```rust
// Prevent privilege escalation
permissions_boundary: Some("arn:aws:iam::123456789012:policy/MaxPermissions".into())
```

### 4. Rotate Access Keys

```rust
// Create new key
let new_key = iam.create_access_key(CreateAccessKeyRequest {
    user_name: "alice".to_string(),
}).await?;

// Update application with new key

// Deactivate old key
iam.update_access_key(UpdateAccessKeyRequest {
    user_name: "alice".to_string(),
    access_key_id: old_key_id.to_string(),
    status: "Inactive".to_string(),
}).await?;

// After verification, delete old key
iam.delete_access_key("alice", &old_key_id).await?;
```

### 5. Enable MFA for Sensitive Operations

```rust
// Require MFA for production access
let policy_with_mfa = r#"{
    "Version": "2012-10-17",
    "Statement": [{
        "Effect": "Allow",
        "Action": "*",
        "Resource": "*",
        "Condition": {
            "Bool": {
                "aws:MultiFactorAuthPresent": "true"
            }
        }
    }]
}"#;
```

## Error Handling

```rust
use wami::AmiError;

match iam.create_user(request).await {
    Ok(response) => println!("Created: {:?}", response.data),
    Err(AmiError::ResourceExists { resource }) => {
        println!("User already exists: {}", resource);
    }
    Err(AmiError::LimitExceeded { message }) => {
        println!("Quota exceeded: {}", message);
    }
    Err(AmiError::InvalidParameter { message }) => {
        println!("Invalid input: {}", message);
    }
    Err(e) => println!("Error: {:?}", e),
}
```

## Complete Example

```rust
use wami::{
    MemoryIamClient, CreateUserRequest, CreateRoleRequest,
    CreatePolicyRequest, CreateAccessKeyRequest, Tag,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = wami::create_memory_store();
    let mut iam = MemoryIamClient::new(store);
    
    // 1. Create user
    let user = iam.create_user(CreateUserRequest {
        user_name: "data-scientist".to_string(),
        path: Some("/teams/ml/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            Tag { key: "Team".into(), value: "ML".into() },
        ]),
    }).await?;
    println!("✓ User: {}", user.data.as_ref().unwrap().arn);
    
    // 2. Create policy
    let policy = iam.create_policy(CreatePolicyRequest {
        policy_name: "MLAccess".to_string(),
        policy_document: r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:*", "sagemaker:*"],
                "Resource": "*"
            }]
        }"#.to_string(),
        path: Some("/policies/ml/".to_string()),
        description: Some("ML team access".to_string()),
        tags: None,
    }).await?;
    let policy_arn = policy.data.as_ref().unwrap().arn.clone();
    println!("✓ Policy: {}", policy_arn);
    
    // 3. Attach policy to user
    iam.attach_user_policy("data-scientist", &policy_arn).await?;
    println!("✓ Attached policy to user");
    
    // 4. Create access keys
    let keys = iam.create_access_key(CreateAccessKeyRequest {
        user_name: "data-scientist".to_string(),
    }).await?;
    let key = keys.data.unwrap();
    println!("✓ Access Key: {}", key.access_key_id);
    
    // 5. Create role
    let role = iam.create_role(CreateRoleRequest {
        role_name: "MLServiceRole".to_string(),
        assume_role_policy_document: r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {"Service": "sagemaker.amazonaws.com"},
                "Action": "sts:AssumeRole"
            }]
        }"#.to_string(),
        path: Some("/roles/ml/".to_string()),
        description: Some("SageMaker service role".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: None,
    }).await?;
    println!("✓ Role: {}", role.data.unwrap().arn);
    
    Ok(())
}
```

## Next Steps

- **[STS Guide](STS_GUIDE.md)** - Temporary credentials
- **[SSO Admin](SSO_ADMIN_GUIDE.md)** - SSO configuration
- **[Multi-Tenant](MULTI_TENANT_GUIDE.md)** - Tenant isolation
- **[Examples](EXAMPLES.md)** - More code samples

## API Reference

Full API documentation: `cargo doc --open`

## Support

Questions? Open an issue on [GitHub](https://github.com/lsh0x/wami/issues).

