# API Reference

Quick reference for WAMI APIs.

## Client Initialization

```rust
// IAM Client
let store = wami::create_memory_store();
let mut iam = MemoryIamClient::new(store);

// STS Client
let mut sts = MemoryStsClient::new(store);

// SSO Admin Client
let mut sso = MemorySsoAdminClient::new(store);

// Tenant Client
let principal = "admin".to_string();
let mut tenant = TenantClient::new(store, principal);
```

## IAM Operations

### Users

| Operation | Method | Returns |
|-----------|--------|---------|
| Create user | `iam.create_user(request)` | `User` |
| Get user | `iam.get_user(user_name)` | `Option<User>` |
| List users | `iam.list_users(path, marker)` | `Vec<User>` |
| Update user | `iam.update_user(request)` | `User` |
| Delete user | `iam.delete_user(user_name)` | `()` |

### Roles

| Operation | Method | Returns |
|-----------|--------|---------|
| Create role | `iam.create_role(request)` | `Role` |
| Get role | `iam.get_role(role_name)` | `Option<Role>` |
| List roles | `iam.list_roles(path, marker)` | `Vec<Role>` |
| Update role | `iam.update_role(request)` | `Role` |
| Delete role | `iam.delete_role(role_name)` | `()` |
| Attach policy | `iam.attach_role_policy(role, arn)` | `()` |
| Detach policy | `iam.detach_role_policy(role, arn)` | `()` |

### Policies

| Operation | Method | Returns |
|-----------|--------|---------|
| Create policy | `iam.create_policy(request)` | `Policy` |
| Get policy | `iam.get_policy(arn)` | `Option<Policy>` |
| List policies | `iam.list_policies(...)` | `Vec<Policy>` |
| Delete policy | `iam.delete_policy(arn)` | `()` |
| Attach to user | `iam.attach_user_policy(user, arn)` | `()` |
| Detach from user | `iam.detach_user_policy(user, arn)` | `()` |

### Groups

| Operation | Method | Returns |
|-----------|--------|---------|
| Create group | `iam.create_group(request)` | `Group` |
| Get group | `iam.get_group(group_name)` | `Option<Group>` |
| List groups | `iam.list_groups(path, marker)` | `Vec<Group>` |
| Delete group | `iam.delete_group(group_name)` | `()` |
| Add user | `iam.add_user_to_group(user, group)` | `()` |
| Remove user | `iam.remove_user_from_group(user, group)` | `()` |

### Access Keys

| Operation | Method | Returns |
|-----------|--------|---------|
| Create key | `iam.create_access_key(request)` | `AccessKey` |
| List keys | `iam.list_access_keys(user, ...)` | `Vec<AccessKey>` |
| Update key | `iam.update_access_key(request)` | `()` |
| Delete key | `iam.delete_access_key(user, key_id)` | `()` |
| Get last used | `iam.get_access_key_last_used(key_id)` | `Option<DateTime>` |

### Passwords

| Operation | Method | Returns |
|-----------|--------|---------|
| Create login | `iam.create_login_profile(request)` | `LoginProfile` |
| Get login | `iam.get_login_profile(user)` | `Option<LoginProfile>` |
| Update login | `iam.update_login_profile(request)` | `()` |
| Delete login | `iam.delete_login_profile(user)` | `()` |

### MFA Devices

| Operation | Method | Returns |
|-----------|--------|---------|
| Enable MFA | `iam.enable_mfa_device(request)` | `()` |
| Deactivate MFA | `iam.deactivate_mfa_device(user, serial)` | `()` |
| List MFA | `iam.list_mfa_devices(user, ...)` | `Vec<MfaDevice>` |
| Resync MFA | `iam.resync_mfa_device(request)` | `()` |

## STS Operations

| Operation | Method | Returns |
|-----------|--------|---------|
| Assume role | `sts.assume_role(request)` | `Credentials` |
| Assume role (SAML) | `sts.assume_role_with_saml(request)` | `Credentials` |
| Assume role (WebID) | `sts.assume_role_with_web_identity(request)` | `Credentials` |
| Get session token | `sts.get_session_token(request)` | `Credentials` |
| Get federation token | `sts.get_federation_token(request)` | `Credentials` |
| Get caller identity | `sts.get_caller_identity()` | `CallerIdentity` |
| Get access key info | `sts.get_access_key_info(key_id)` | `AccessKeyInfo` |

## SSO Admin Operations

### Permission Sets

| Operation | Method | Returns |
|-----------|--------|---------|
| Create | `sso.create_permission_set(request)` | `PermissionSet` |
| Describe | `sso.describe_permission_set(instance, arn)` | `PermissionSet` |
| List | `sso.list_permission_sets(instance, ...)` | `Vec<String>` |
| Update | `sso.update_permission_set(request)` | `()` |
| Delete | `sso.delete_permission_set(instance, arn)` | `()` |

### Managed Policies

| Operation | Method | Returns |
|-----------|--------|---------|
| Attach | `sso.attach_managed_policy_to_permission_set(...)` | `()` |
| List | `sso.list_managed_policies_in_permission_set(...)` | `Vec<Policy>` |
| Detach | `sso.detach_managed_policy_from_permission_set(...)` | `()` |

### Inline Policies

| Operation | Method | Returns |
|-----------|--------|---------|
| Put | `sso.put_inline_policy_to_permission_set(...)` | `()` |
| Get | `sso.get_inline_policy_for_permission_set(...)` | `Option<String>` |
| Delete | `sso.delete_inline_policy_from_permission_set(...)` | `()` |

### Account Assignments

| Operation | Method | Returns |
|-----------|--------|---------|
| Create | `sso.create_account_assignment(request)` | `Assignment` |
| List | `sso.list_account_assignments(...)` | `Vec<Assignment>` |
| Delete | `sso.delete_account_assignment(request)` | `()` |

## Tenant Operations

| Operation | Method | Returns |
|-----------|--------|---------|
| Create root | `tenant.create_root_tenant(request)` | `Tenant` |
| Create sub-tenant | `tenant.create_sub_tenant(parent, request)` | `Tenant` |
| Get tenant | `tenant.get_tenant(id)` | `Option<Tenant>` |
| List children | `tenant.list_child_tenants(id)` | `Vec<Tenant>` |
| Delete cascade | `tenant.delete_tenant_cascade(id)` | `()` |

## Data Structures

### User

```rust
pub struct User {
    pub user_name: String,
    pub user_id: String,
    pub arn: String,
    pub path: String,
    pub create_date: DateTime<Utc>,
    pub password_last_used: Option<DateTime<Utc>>,
    pub permissions_boundary: Option<String>,
    pub tags: Vec<Tag>,
    pub tenant_id: Option<TenantId>,
}
```

### Role

```rust
pub struct Role {
    pub role_name: String,
    pub role_id: String,
    pub arn: String,
    pub path: String,
    pub assume_role_policy_document: String,
    pub create_date: DateTime<Utc>,
    pub description: Option<String>,
    pub max_session_duration: u32,
    pub tenant_id: Option<TenantId>,
}
```

### Policy

```rust
pub struct Policy {
    pub policy_name: String,
    pub policy_id: String,
    pub arn: String,
    pub path: String,
    pub default_version_id: String,
    pub attachment_count: u32,
    pub create_date: DateTime<Utc>,
    pub update_date: DateTime<Utc>,
    pub tenant_id: Option<TenantId>,
}
```

### Credentials

```rust
pub struct Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub expiration: DateTime<Utc>,
}
```

### Tenant

```rust
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub tenant_type: TenantType,
    pub status: TenantStatus,
    pub parent_id: Option<TenantId>,
    pub quotas: TenantQuotas,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Request Structures

### CreateUserRequest

```rust
pub struct CreateUserRequest {
    pub user_name: String,
    pub path: Option<String>,
    pub permissions_boundary: Option<String>,
    pub tags: Option<Vec<Tag>>,
}
```

### CreateRoleRequest

```rust
pub struct CreateRoleRequest {
    pub role_name: String,
    pub assume_role_policy_document: String,
    pub path: Option<String>,
    pub description: Option<String>,
    pub max_session_duration: Option<u32>,
    pub permissions_boundary: Option<String>,
    pub tags: Option<Vec<Tag>>,
}
```

### AssumeRoleRequest

```rust
pub struct AssumeRoleRequest {
    pub role_arn: String,
    pub role_session_name: String,
    pub duration_seconds: Option<u32>,
    pub external_id: Option<String>,
    pub policy: Option<String>,
}
```

## Error Types

```rust
pub enum AmiError {
    ResourceNotFound { resource: String },
    ResourceExists { resource: String },
    InvalidParameter { message: String },
    LimitExceeded { message: String },
    Unauthorized { message: String },
    InternalError { message: String },
    StoreError { source: Box<dyn Error> },
}
```

## Response Wrapper

```rust
pub struct Response<T> {
    pub data: Option<T>,
    pub error: Option<ResponseError>,
}

impl<T> Response<T> {
    pub fn success(data: T) -> Self;
    pub fn error(code: String, message: String) -> Self;
}
```

## Provider Traits

### CloudProvider

```rust
pub trait CloudProvider: Send + Sync {
    fn name(&self) -> &str;
    fn generate_resource_identifier(&self, type: &str, name: &str, path: Option<&str>) -> String;
    fn generate_account_id(&self) -> String;
    fn max_session_duration(&self) -> u32;
    fn validate_session_duration(&self, duration: u32) -> Result<()>;
    fn max_users(&self) -> Option<u32>;
    fn max_roles(&self) -> Option<u32>;
    fn max_policies(&self) -> Option<u32>;
}
```

### Store

```rust
pub trait Store: Send + Sync {
    type IamStore: IamStore;
    type StsStore: StsStore;
    type SsoAdminStore: SsoAdminStore;
    type TenantStore: TenantStore;
    
    fn cloud_provider(&self) -> &dyn CloudProvider;
    async fn iam_store(&mut self) -> Result<&mut Self::IamStore>;
    async fn sts_store(&mut self) -> Result<&mut Self::StsStore>;
    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore>;
    async fn tenant_store(&mut self) -> Result<&mut Self::TenantStore>;
}
```

## Full API Documentation

For complete API documentation with all types, methods, and examples:

```bash
cargo doc --open
```

Or visit [docs.rs/wami](https://docs.rs/wami)

## Support

Questions? Open an issue on [GitHub](https://github.com/lsh0x/wami/issues).

