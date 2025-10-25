# AMI.rs

AWS IAM, STS, and SSO Admin operations library for Rust

[![GitHub last commit](https://img.shields.io/github/last-commit/lsh0x/AMI.rs)](https://github.com/lsh0x/AMI.rs/commits/main)
[![CI](https://github.com/lsh0x/AMI.rs/workflows/CI/badge.svg)](https://github.com/lsh0x/AMI.rs/actions)
[![Codecov](https://codecov.io/gh/lsh0x/AMI.rs/branch/main/graph/badge.svg)](https://codecov.io/gh/lsh0x/AMI.rs)
[![Docs](https://docs.rs/ami/badge.svg)](https://docs.rs/ami)
[![Crates.io](https://img.shields.io/crates/v/ami.svg)](https://crates.io/crates/ami)
[![crates.io](https://img.shields.io/crates/d/ami)](https://crates.io/crates/ami)

## Overview

AMI.rs is a comprehensive Rust library that provides easy-to-use interfaces for AWS Identity and Access Management (IAM), Security Token Service (STS), and Single Sign-On Admin operations. This library abstracts the complexity of AWS SDK calls and provides a clean, type-safe API for managing AWS identities and permissions.

**Key Features:**
- 🔐 **Complete IAM Management** - Users, groups, roles, policies, and access controls
- 🔑 **Temporary Credentials** - STS operations for secure, time-limited access
- 🏢 **SSO Administration** - Permission sets, assignments, and federation
- 💾 **In-Memory Storage** - Fast, lightweight implementation for testing and development
- 📚 **Comprehensive Documentation** - Detailed rustdoc with examples for every operation
- ⚡ **Async API** - Built on Tokio for high-performance async operations
- 🛡️ **Type-Safe** - Strongly typed requests and responses

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [IAM Operations](#iam-operations)
- [STS Operations](#sts-operations)
- [SSO Admin Operations](#sso-admin-operations)
- [Account ID Management](#account-id-management)
- [AWS Environment Variables](#aws-environment-variables)
- [Contributing](#contributing)
- [License](#license)

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ami = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

---

## Quick Start

```rust
use ami::{MemoryIamClient, MemoryStsClient, MemorySsoAdminClient};
use ami::{CreateUserRequest, AssumeRoleRequest, CreatePermissionSetRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Create shared store with auto-generated account ID
    let store = ami::create_memory_store();
    let account_id = ami::get_account_id_from_store(&store);
    println!("Using AWS account ID: {}", account_id);
    
    // Initialize clients
    let mut iam_client = MemoryIamClient::new(store.clone());
    let mut sts_client = MemoryStsClient::new(store.clone());
    let mut sso_client = MemorySsoAdminClient::new(store);
    
    // IAM: Create a user
    let user_request = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/engineering/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let user = iam_client.create_user(user_request).await?;
    println!("Created user: {}", user.data.unwrap().arn);
    
    // STS: Get caller identity
    let identity = sts_client.get_caller_identity().await?;
    println!("Caller: {}", identity.data.unwrap().arn);
    
    // SSO: Create permission set
    let ps_request = CreatePermissionSetRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
        name: "DeveloperAccess".to_string(),
        description: Some("Developer permissions".to_string()),
        session_duration: Some("PT8H".to_string()),
        relay_state: None,
    };
    let permission_set = sso_client.create_permission_set(ps_request).await?;
    println!("Created permission set: {}", permission_set.data.unwrap().permission_set_arn);
    
    Ok(())
}
```

---

## IAM Operations

AWS Identity and Access Management (IAM) operations for managing users, groups, roles, and policies.

### Example: User Management

```rust
use ami::{MemoryIamClient, CreateUserRequest, CreateAccessKeyRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = ami::create_memory_store();
    let mut iam_client = MemoryIamClient::new(store);
    
    // Create a user
    let user_request = CreateUserRequest {
        user_name: "developer".to_string(),
        path: Some("/engineering/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let user = iam_client.create_user(user_request).await?;
    println!("Created user: {}", user.data.unwrap().arn);
    
    // Create access keys
    let key_request = CreateAccessKeyRequest {
        user_name: "developer".to_string(),
    };
    let access_key = iam_client.create_access_key(key_request).await?;
    let key = access_key.data.unwrap();
    println!("Access Key ID: {}", key.access_key_id);
    println!("Secret Key: {}", key.secret_access_key.unwrap());
    
    Ok(())
}
```

### Available IAM Operations

<details>
<summary><strong>👤 Users</strong></summary>

- `create_user` - Create a new IAM user
- `delete_user` - Delete an IAM user
- `get_user` - Retrieve user information
- `update_user` - Update user properties
- `list_users` - List all users
- `list_user_tags` - List tags for a user

</details>

<details>
<summary><strong>🔑 Access Keys</strong></summary>

- `create_access_key` - Create access keys for a user
- `delete_access_key` - Delete access keys
- `update_access_key` - Update access key status
- `list_access_keys` - List user's access keys
- `get_access_key_last_used` - Get last used information

</details>

<details>
<summary><strong>🔐 Passwords</strong></summary>

- `create_login_profile` - Create console login profile
- `update_login_profile` - Update login profile
- `delete_login_profile` - Delete login profile
- `get_login_profile` - Get login profile information

</details>

<details>
<summary><strong>📱 MFA Devices</strong></summary>

- `enable_mfa_device` - Enable MFA device
- `deactivate_mfa_device` - Deactivate MFA device
- `list_mfa_devices` - List MFA devices
- `resync_mfa_device` - Resync MFA device

</details>

<details>
<summary><strong>👥 Groups</strong></summary>

- `create_group` - Create a new group
- `update_group` - Update group properties
- `delete_group` - Delete a group
- `get_group` - Get group information
- `list_groups` - List all groups
- `list_groups_for_user` - List groups for a user
- `add_user_to_group` - Add user to group
- `remove_user_from_group` - Remove user from group
- `attach_group_policy` - Attach policy to group
- `detach_group_policy` - Detach policy from group

</details>

<details>
<summary><strong>🎭 Roles</strong></summary>

- `create_role` - Create a new role
- `update_role` - Update role properties
- `delete_role` - Delete a role
- `get_role` - Get role information
- `list_roles` - List all roles
- `attach_role_policy` - Attach managed policy
- `detach_role_policy` - Detach managed policy
- `update_assume_role_policy` - Update trust policy
- `create_instance_profile` - Create instance profile
- `add_role_to_instance_profile` - Add role to instance profile

</details>

<details>
<summary><strong>📋 Policies</strong></summary>

- `create_policy` - Create managed policy
- `delete_policy` - Delete managed policy
- `get_policy` - Get managed policy
- `list_policies` - List managed policies
- `attach_user_policy` - Attach policy to user
- `detach_user_policy` - Detach policy from user
- `put_user_policy` - Put user inline policy
- `get_user_policy` - Get user inline policy

</details>

<details>
<summary><strong>🔏 Permissions Boundaries & Policy Evaluation</strong></summary>

- `put_user_permissions_boundary` - Set user permissions boundary
- `delete_user_permissions_boundary` - Delete user permissions boundary
- `simulate_custom_policy` - Simulate custom policy
- `simulate_principal_policy` - Simulate principal policy

</details>

<details>
<summary><strong>🌐 Identity Providers, Certificates & Tags</strong></summary>

- SAML & OIDC provider management
- Server certificate management
- Signing certificate management
- Resource tagging operations
- Credential and access reports

</details>

---

## STS Operations

AWS Security Token Service (STS) operations for requesting temporary, limited-privilege credentials.

### Example: Assume Role

```rust
use ami::{MemoryStsClient, AssumeRoleRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = ami::create_memory_store();
    let mut sts_client = MemoryStsClient::new(store);
    
    // Assume a role
    let request = AssumeRoleRequest {
        role_arn: "arn:aws:iam::123456789012:role/DataScientist".to_string(),
        role_session_name: "analytics-session".to_string(),
        duration_seconds: Some(3600),
        external_id: None,
        policy: None,
    };
    
    let response = sts_client.assume_role(request).await?;
    let credentials = response.data.unwrap();
    
    println!("Access Key: {}", credentials.access_key_id);
    println!("Secret Key: {}", credentials.secret_access_key);
    println!("Session Token: {}", credentials.session_token);
    println!("Expires: {}", credentials.expiration);
    
    // Get caller identity
    let identity = sts_client.get_caller_identity().await?;
    let id = identity.data.unwrap();
    println!("Caller ARN: {}", id.arn);
    
    Ok(())
}
```

### Available STS Operations

<details>
<summary><strong>🔑 Temporary Credentials</strong></summary>

- `assume_role` - Assume a role and get temporary credentials
- `assume_role_with_saml` - Assume role with SAML assertion
- `assume_role_with_web_identity` - Assume role with web identity token
- `get_federation_token` - Get federation token for federated users
- `get_session_token` - Get session token for MFA-authenticated users
- `decode_authorization_message` - Decode authorization failure messages

</details>

<details>
<summary><strong>🔍 Identity Inspection</strong></summary>

- `get_caller_identity` - Get details about the calling identity
- `get_access_key_info` - Get information about an access key

</details>

---

## SSO Admin Operations

AWS Single Sign-On Admin operations for managing permission sets, account assignments, and SSO instances.

### Example: Permission Sets & Assignments

```rust
use ami::{MemorySsoAdminClient, CreatePermissionSetRequest, CreateAccountAssignmentRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = ami::create_memory_store();
    let mut sso_client = MemorySsoAdminClient::new(store);
    
    // Create a permission set
    let ps_request = CreatePermissionSetRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
        name: "DataScientistAccess".to_string(),
        description: Some("Permissions for data scientists".to_string()),
        session_duration: Some("PT8H".to_string()),
        relay_state: None,
    };
    
    let ps_response = sso_client.create_permission_set(ps_request).await?;
    let permission_set = ps_response.data.unwrap();
    println!("Created permission set: {}", permission_set.permission_set_arn);
    
    // Create an account assignment
    let assignment_request = CreateAccountAssignmentRequest {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
        target_id: "123456789012".to_string(),
        target_type: "AWS_ACCOUNT".to_string(),
        permission_set_arn: permission_set.permission_set_arn,
        principal_type: "USER".to_string(),
        principal_id: "user-id-12345".to_string(),
    };
    
    let assignment_response = sso_client.create_account_assignment(assignment_request).await?;
    println!("Created assignment: {:?}", assignment_response.data);
    
    Ok(())
}
```

### Available SSO Admin Operations

<details>
<summary><strong>📦 Permission Sets</strong></summary>

- `create_permission_set` - Create permission set
- `update_permission_set` - Update permission set
- `delete_permission_set` - Delete permission set
- `describe_permission_set` - Describe permission set
- `list_permission_sets` - List permission sets
- `attach_managed_policy_to_permission_set` - Attach managed policy
- `detach_managed_policy_from_permission_set` - Detach managed policy

</details>

<details>
<summary><strong>🔗 Account Assignments</strong></summary>

- `create_account_assignment` - Create account assignment
- `delete_account_assignment` - Delete account assignment
- `list_account_assignments` - List account assignments
- `describe_account_assignment_creation_status` - Describe creation status

</details>

<details>
<summary><strong>🏢 Instances & Applications</strong></summary>

- `list_instances` - List SSO instances
- `list_applications` - List applications
- `create_trusted_token_issuer` - Create trusted token issuer
- `list_trusted_token_issuers` - List trusted token issuers

</details>

---

## Account ID Management

### Auto-Generated Account ID

```rust
use ami::MemoryIamClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let store = ami::create_memory_store();
    let account_id = ami::get_account_id_from_store(&store);
    println!("Using AWS account ID: {}", account_id);
    
    let mut iam_client = MemoryIamClient::new(store);
    
    // All ARNs will use the auto-generated account ID
    let user_request = ami::CreateUserRequest {
        user_name: "test-user".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let user = iam_client.create_user(user_request).await?;
    println!("Created user ARN: {}", user.data.unwrap().arn);
    
    Ok(())
}
```

### Custom Account ID

```rust
use ami::MemoryIamClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a specific account ID
    let store = ami::create_memory_store_with_account_id("123456789012".to_string());
    let mut iam_client = MemoryIamClient::new(store);
    
    // All ARNs will use the specified account ID
    let user_request = ami::CreateUserRequest {
        user_name: "my-user".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let user = iam_client.create_user(user_request).await?;
    println!("User ARN: {}", user.data.unwrap().arn);
    // Output: arn:aws:iam::123456789012:user/my-user
    
    Ok(())
}
```

### Retrieving Account ID

AMI.rs automatically generates realistic 12-digit AWS account IDs for each instance:

- **Auto-generate**: Use `create_memory_store()` for a random account ID
- **Custom ID**: Use `create_memory_store_with_account_id("123456789012")` for a specific ID
- **Retrieve ID**: Use `get_account_id_from_store(&store)` or `client.account_id().await?`
- **Logging**: Enable logging with `env_logger::init()` to see account ID generation

All ARNs (users, groups, roles, policies) will use the same account ID consistently across IAM, STS, and SSO Admin operations.

---

## AWS Environment Variables

AMI.rs provides AWS environment variables for compatibility with AWS CLI and other tools.

### Example

```rust
use ami::create_memory_store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let store = create_memory_store();
    
    // Print environment variables to console
    ami::print_aws_environment_variables(&store);
    
    // Or get them programmatically
    let env_vars = store.aws_environment_variables();
    println!("AWS_ACCOUNT_ID: {}", env_vars["AWS_ACCOUNT_ID"]);
    println!("AWS_REGION: {}", env_vars["AWS_REGION"]);
    
    Ok(())
}
```

### Output

```
INFO ami::store::in_memory: Generated AWS account ID: 847392847392

AWS Environment Variables:
  export AWS_ACCOUNT_ID=847392847392
  export AWS_DEFAULT_REGION=us-east-1
  export AWS_REGION=us-east-1
  export AWS_PROFILE=default

To use with AWS CLI or other tools, run:
  export AWS_ACCOUNT_ID=847392847392
  export AWS_DEFAULT_REGION=us-east-1
```

### Export for Shell

```bash
export AWS_ACCOUNT_ID=847392847392
export AWS_DEFAULT_REGION=us-east-1
export AWS_REGION=us-east-1
export AWS_PROFILE=default
```

---

## Documentation

For detailed API documentation with examples, run:

```bash
cargo doc --open
```

Or visit the online documentation at [docs.rs/ami](https://docs.rs/ami).

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/lsh0x/AMI.rs.git
   cd AMI.rs
   ```

2. **Install Git hooks** (recommended to catch issues before committing):
   ```bash
   git config core.hooksPath .githooks
   ```
   
   This will automatically run `cargo fmt` and `cargo clippy` checks before each commit.
   See [.githooks/README.md](.githooks/README.md) for more details.

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Check formatting:**
   ```bash
   cargo fmt --all
   ```

5. **Run clippy:**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

### Code Quality Standards

All pull requests must:
- ✅ Pass all tests
- ✅ Have no clippy warnings
- ✅ Be properly formatted with `rustfmt`
- ✅ Include documentation for public APIs
- ✅ Add tests for new functionality

---

## Support

For questions, issues, or feature requests, please open an issue on [GitHub](https://github.com/lsh0x/AMI.rs/issues).
