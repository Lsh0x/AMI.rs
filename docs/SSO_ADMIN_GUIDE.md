# SSO Admin Operations Guide

Complete guide to AWS Single Sign-On Administration with WAMI.

## Overview

WAMI provides AWS SSO Admin operations for managing:
- **Permission Sets** - Collections of permissions for SSO users
- **Account Assignments** - Assign permission sets to users/groups in accounts
- **Managed Policies** - Attach AWS managed policies to permission sets
- **Inline Policies** - Create custom policies for permission sets

## Quick Start

```rust
use wami::{MemorySsoAdminClient, CreatePermissionSetRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = wami::create_memory_store();
    let mut sso = MemorySsoAdminClient::new(store);
    
    // Create permission set
    let ps = sso.create_permission_set(CreatePermissionSetRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
        name: "DeveloperAccess".to_string(),
        description: Some("Permissions for developers".to_string()),
        session_duration: Some("PT8H".to_string()),
        relay_state: None,
    }).await?;
    
    println!("Created: {}", ps.data.unwrap().permission_set_arn);
    Ok(())
}
```

## Permission Sets

### Create Permission Set

```rust
use wami::{MemorySsoAdminClient, CreatePermissionSetRequest};

let mut sso = MemorySsoAdminClient::new(store);

let ps = sso.create_permission_set(CreatePermissionSetRequest {
    instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
    name: "DataScientistAccess".to_string(),
    description: Some("Full access to data science tools".to_string()),
    session_duration: Some("PT12H".to_string()), // 12 hours
    relay_state: Some("https://console.aws.amazon.com".to_string()),
}).await?;

let permission_set_arn = ps.data.unwrap().permission_set_arn;
println!("Created: {}", permission_set_arn);
```

### Describe Permission Set

```rust
let ps = sso.describe_permission_set(
    "arn:aws:sso:::instance/ssoins-1234",
    &permission_set_arn,
).await?;

if let Some(data) = ps.data {
    println!("Name: {}", data.name);
    println!("Description: {:?}", data.description);
    println!("Session Duration: {:?}", data.session_duration);
}
```

### List Permission Sets

```rust
let response = sso.list_permission_sets(
    "arn:aws:sso:::instance/ssoins-1234",
    None, // No pagination
    None, // No limit
).await?;

for ps_arn in response.data.unwrap().permission_sets {
    println!("- {}", ps_arn);
}
```

### Update Permission Set

```rust
use wami::UpdatePermissionSetRequest;

sso.update_permission_set(UpdatePermissionSetRequest {
    instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
    permission_set_arn: permission_set_arn.clone(),
    description: Some("Updated description".to_string()),
    session_duration: Some("PT4H".to_string()), // 4 hours
    relay_state: None,
}).await?;
```

### Delete Permission Set

```rust
sso.delete_permission_set(
    "arn:aws:sso:::instance/ssoins-1234",
    &permission_set_arn,
).await?;
```

## Managed Policies

### Attach Managed Policy

```rust
use wami::AttachManagedPolicyToPermissionSetRequest;

sso.attach_managed_policy_to_permission_set(
    AttachManagedPolicyToPermissionSetRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
        permission_set_arn: permission_set_arn.clone(),
        managed_policy_arn: "arn:aws:iam::aws:policy/ReadOnlyAccess".to_string(),
    }
).await?;

println!("Attached ReadOnlyAccess policy");
```

### List Attached Managed Policies

```rust
let policies = sso.list_managed_policies_in_permission_set(
    "arn:aws:sso:::instance/ssoins-1234",
    &permission_set_arn,
    None,
    None,
).await?;

println!("Attached policies:");
for policy in policies.data.unwrap().attached_managed_policies {
    println!("- {} ({})", policy.name, policy.arn);
}
```

### Detach Managed Policy

```rust
use wami::DetachManagedPolicyFromPermissionSetRequest;

sso.detach_managed_policy_from_permission_set(
    DetachManagedPolicyFromPermissionSetRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
        permission_set_arn: permission_set_arn.clone(),
        managed_policy_arn: "arn:aws:iam::aws:policy/ReadOnlyAccess".to_string(),
    }
).await?;
```

## Inline Policies

### Put Inline Policy

```rust
use wami::PutInlinePolicyToPermissionSetRequest;

let policy_document = r#"{
    "Version": "2012-10-17",
    "Statement": [{
        "Effect": "Allow",
        "Action": [
            "s3:GetObject",
            "s3:PutObject"
        ],
        "Resource": "arn:aws:s3:::my-bucket/*"
    }]
}"#;

sso.put_inline_policy_to_permission_set(
    PutInlinePolicyToPermissionSetRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
        permission_set_arn: permission_set_arn.clone(),
        inline_policy: policy_document.to_string(),
    }
).await?;

println!("Added inline policy");
```

### Get Inline Policy

```rust
let response = sso.get_inline_policy_for_permission_set(
    "arn:aws:sso:::instance/ssoins-1234",
    &permission_set_arn,
).await?;

if let Some(policy) = response.data {
    println!("Inline policy:\n{}", policy.inline_policy);
}
```

### Delete Inline Policy

```rust
use wami::DeleteInlinePolicyFromPermissionSetRequest;

sso.delete_inline_policy_from_permission_set(
    DeleteInlinePolicyFromPermissionSetRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
        permission_set_arn: permission_set_arn.clone(),
    }
).await?;
```

## Account Assignments

### Create Account Assignment

```rust
use wami::CreateAccountAssignmentRequest;

let assignment = sso.create_account_assignment(CreateAccountAssignmentRequest {
    instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
    target_id: "123456789012".to_string(), // AWS Account ID
    target_type: "AWS_ACCOUNT".to_string(),
    permission_set_arn: permission_set_arn.clone(),
    principal_type: "USER".to_string(), // or "GROUP"
    principal_id: "user-id-12345".to_string(),
}).await?;

println!("Created assignment: {:?}", assignment.data);
```

### List Account Assignments

```rust
let assignments = sso.list_account_assignments(
    "arn:aws:sso:::instance/ssoins-1234",
    "123456789012",
    &permission_set_arn,
    None,
    None,
).await?;

for assignment in assignments.data.unwrap().account_assignments {
    println!("Principal: {} ({})", assignment.principal_id, assignment.principal_type);
}
```

### Delete Account Assignment

```rust
use wami::DeleteAccountAssignmentRequest;

sso.delete_account_assignment(DeleteAccountAssignmentRequest {
    instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
    target_id: "123456789012".to_string(),
    target_type: "AWS_ACCOUNT".to_string(),
    permission_set_arn: permission_set_arn.clone(),
    principal_type: "USER".to_string(),
    principal_id: "user-id-12345".to_string(),
}).await?;
```

## Complete Example

```rust
use wami::{
    MemorySsoAdminClient,
    CreatePermissionSetRequest,
    AttachManagedPolicyToPermissionSetRequest,
    CreateAccountAssignmentRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = wami::create_memory_store();
    let mut sso = MemorySsoAdminClient::new(store);
    
    let instance_arn = "arn:aws:sso:::instance/ssoins-1234".to_string();
    
    // 1. Create permission set
    let ps = sso.create_permission_set(CreatePermissionSetRequest {
        instance_arn: instance_arn.clone(),
        name: "EngineeringAccess".to_string(),
        description: Some("Engineering team access".to_string()),
        session_duration: Some("PT8H".to_string()),
        relay_state: None,
    }).await?;
    let ps_arn = ps.data.unwrap().permission_set_arn;
    println!("✓ Created permission set: {}", ps_arn);
    
    // 2. Attach managed policy
    sso.attach_managed_policy_to_permission_set(
        AttachManagedPolicyToPermissionSetRequest {
            instance_arn: instance_arn.clone(),
            permission_set_arn: ps_arn.clone(),
            managed_policy_arn: "arn:aws:iam::aws:policy/PowerUserAccess".to_string(),
        }
    ).await?;
    println!("✓ Attached PowerUserAccess policy");
    
    // 3. Add inline policy
    use wami::PutInlinePolicyToPermissionSetRequest;
    sso.put_inline_policy_to_permission_set(
        PutInlinePolicyToPermissionSetRequest {
            instance_arn: instance_arn.clone(),
            permission_set_arn: ps_arn.clone(),
            inline_policy: r#"{
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": "Allow",
                    "Action": "s3:*",
                    "Resource": "*"
                }]
            }"#.to_string(),
        }
    ).await?;
    println!("✓ Added inline policy");
    
    // 4. Create account assignment
    sso.create_account_assignment(CreateAccountAssignmentRequest {
        instance_arn: instance_arn.clone(),
        target_id: "123456789012".to_string(),
        target_type: "AWS_ACCOUNT".to_string(),
        permission_set_arn: ps_arn.clone(),
        principal_type: "GROUP".to_string(),
        principal_id: "group-engineering".to_string(),
    }).await?;
    println!("✓ Assigned to engineering group");
    
    // 5. List all assignments
    let assignments = sso.list_account_assignments(
        &instance_arn,
        "123456789012",
        &ps_arn,
        None,
        None,
    ).await?;
    
    println!("\n📋 Account Assignments:");
    for assignment in assignments.data.unwrap().account_assignments {
        println!("  - {} ({}) in account {}",
            assignment.principal_id,
            assignment.principal_type,
            assignment.account_id
        );
    }
    
    Ok(())
}
```

## Best Practices

### 1. Use Descriptive Names

```rust
// Good
name: "DataScientist-ReadOnly-Access"

// Bad
name: "ps1"
```

### 2. Set Appropriate Session Durations

```rust
// Development: Shorter sessions
session_duration: Some("PT4H".to_string()) // 4 hours

// Production: Longer sessions
session_duration: Some("PT12H".to_string()) // 12 hours
```

### 3. Use Managed Policies When Possible

```rust
// Prefer AWS managed policies
managed_policy_arn: "arn:aws:iam::aws:policy/ReadOnlyAccess"

// Only use inline policies for custom requirements
```

### 4. Document Permission Sets

```rust
description: Some(
    "Engineering team: Full EC2 and S3 access, read-only for other services"
        .to_string()
)
```

### 5. Group Similar Permissions

```rust
// Create permission sets by role/function
"Developer-ReadWrite"
"Analyst-ReadOnly"
"Admin-FullAccess"
```

## Session Duration Format

SSO uses ISO 8601 duration format:

| Format | Duration |
|--------|----------|
| `PT1H` | 1 hour |
| `PT4H` | 4 hours |
| `PT8H` | 8 hours |
| `PT12H` | 12 hours |

## Error Handling

```rust
use wami::AmiError;

match sso.create_permission_set(request).await {
    Ok(response) => println!("Created: {:?}", response.data),
    Err(AmiError::ResourceExists { resource }) => {
        println!("Permission set already exists: {}", resource);
    }
    Err(AmiError::InvalidParameter { message }) => {
        println!("Invalid parameter: {}", message);
    }
    Err(e) => println!("Error: {:?}", e),
}
```

## Next Steps

- **[IAM Guide](IAM_GUIDE.md)** - User, role, policy management
- **[STS Guide](STS_GUIDE.md)** - Temporary credentials
- **[Multi-Tenant](MULTI_TENANT_GUIDE.md)** - Tenant isolation
- **[Examples](EXAMPLES.md)** - More code samples

## Support

Questions? Open an issue on [GitHub](https://github.com/lsh0x/wami/issues).

