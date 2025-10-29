# API Reference

Complete API documentation for WAMI.

## Overview

WAMI's API is organized into two main layers:

1. **Domain Layer** (`wami::*`) - Pure functions and models
2. **Storage Layer** (`store::*`) - Persistence traits and implementations

---

## Domain Layer (wami::*)

### Identity Module (`wami::identity`)

#### User (`wami::identity::user`)

**Model**:
```rust
pub struct User {
    pub user_name: String,
    pub user_id: String,
    pub arn: String,
    pub path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub wami_arn: String,
    pub providers: Vec<ProviderConfig>,
    pub tenant_id: Option<TenantId>,
}
```

**Builder**:
```rust
pub fn build_user(
    user_name: String,
    path: Option<String>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> User
```

#### Group (`wami::identity::group`)

**Model**:
```rust
pub struct Group {
    pub group_name: String,
    pub group_id: String,
    pub arn: String,
    pub path: Option<String>,
    pub created_at: DateTime<Utc>,
    // ...
}
```

**Builder**:
```rust
pub fn build_group(
    group_name: String,
    path: Option<String>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Group
```

#### Role (`wami::identity::role`)

**Model**:
```rust
pub struct Role {
    pub role_name: String,
    pub role_id: String,
    pub arn: String,
    pub assume_role_policy_document: String,
    pub path: Option<String>,
    pub description: Option<String>,
    // ...
}
```

**Builder**:
```rust
pub fn build_role(
    role_name: String,
    assume_role_policy_document: String,
    path: Option<String>,
    description: Option<String>,
    tags: Option<Vec<Tag>>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Role
```

### Credentials Module (`wami::credentials`)

#### Access Key (`wami::credentials::access_key`)

**Model**:
```rust
pub struct AccessKey {
    pub user_name: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub status: AccessKeyStatus,
    pub created_at: DateTime<Utc>,
    // ...
}

pub enum AccessKeyStatus {
    Active,
    Inactive,
}
```

**Builder**:
```rust
pub fn build_access_key(
    user_name: String,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> AccessKey
```

#### MFA Device (`wami::credentials::mfa_device`)

**Model**:
```rust
pub struct MfaDevice {
    pub serial_number: String,
    pub user_name: String,
    pub enable_date: DateTime<Utc>,
    pub arn: String,
    // ...
}
```

#### Login Profile (`wami::credentials::login_profile`)

**Model**:
```rust
pub struct LoginProfile {
    pub user_name: String,
    pub created_at: DateTime<Utc>,
    pub password_reset_required: bool,
    // ...
}
```

### STS Module (`wami::sts`)

#### Session (`wami::sts::session`)

**Model**:
```rust
pub struct StsSession {
    pub session_token: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub expiration: DateTime<Utc>,
    pub status: SessionStatus,
    pub assumed_role_arn: Option<String>,
    // ...
}

pub enum SessionStatus {
    Active,
    Expired,
    Revoked,
}
```

**Builder**:
```rust
pub fn build_session(
    session_token: String,
    access_key_id: String,
    secret_access_key: String,
    duration_seconds: i64,
    assumed_role_arn: Option<String>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> StsSession
```

#### Caller Identity (`wami::sts::identity`)

**Model**:
```rust
pub struct CallerIdentity {
    pub user_id: String,
    pub account: String,
    pub arn: String,
    pub wami_arn: String,
    pub providers: Vec<ProviderConfig>,
}
```

### Tenant Module (`wami::tenant`)

**Model**:
```rust
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub parent_id: Option<TenantId>,
    pub organization: Option<String>,
    pub tenant_type: TenantType,
    pub status: TenantStatus,
    pub quotas: TenantQuotas,
    // ...
}

pub struct TenantId(String);

impl TenantId {
    pub fn root(name: &str) -> Self;
    pub fn child(&self, name: &str) -> Self;
    pub fn parent(&self) -> Option<Self>;
    pub fn depth(&self) -> usize;
    pub fn ancestors(&self) -> Vec<TenantId>;
    pub fn is_descendant_of(&self, other: &TenantId) -> bool;
}

pub enum TenantType {
    Root,
    Enterprise,
    Department,
    Team,
    Project,
    Custom(String),
}

pub enum TenantStatus {
    Active,
    Suspended,
    Pending,
    Deleted,
}
```

---

## Storage Layer (store::*)

### Store Traits

#### WamiStore Composite Trait

```rust
pub trait WamiStore: 
    UserStore + 
    GroupStore + 
    RoleStore + 
    AccessKeyStore + 
    MfaDeviceStore + 
    LoginProfileStore + 
    PolicyStore + 
    ServerCertificateStore + 
    SigningCertificateStore + 
    ServiceCredentialStore + 
    ServiceLinkedRoleStore + 
    CredentialReportStore +
    Send + Sync 
{}
```

#### UserStore

```rust
#[async_trait]
pub trait UserStore: Send + Sync {
    async fn create_user(&mut self, user: User) -> Result<User>;
    async fn get_user(&self, user_name: &str) -> Result<Option<User>>;
    async fn update_user(&mut self, user: User) -> Result<User>;
    async fn delete_user(&mut self, user_name: &str) -> Result<()>;
    async fn list_users(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<User>, bool, Option<String>)>;
    async fn tag_user(&mut self, user_name: &str, tags: Vec<Tag>) -> Result<()>;
    async fn list_user_tags(&self, user_name: &str) -> Result<Vec<Tag>>;
    async fn untag_user(&mut self, user_name: &str, tag_keys: Vec<String>) -> Result<()>;
}
```

#### GroupStore

```rust
#[async_trait]
pub trait GroupStore: Send + Sync {
    async fn create_group(&mut self, group: Group) -> Result<Group>;
    async fn get_group(&self, group_name: &str) -> Result<Option<Group>>;
    async fn update_group(&mut self, group: Group) -> Result<Group>;
    async fn delete_group(&mut self, group_name: &str) -> Result<()>;
    async fn list_groups(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Group>, bool, Option<String>)>;
    async fn add_user_to_group(&mut self, group_name: &str, user_name: &str) -> Result<()>;
    async fn remove_user_from_group(&mut self, group_name: &str, user_name: &str) -> Result<()>;
    async fn list_groups_for_user(&self, user_name: &str) -> Result<Vec<Group>>;
}
```

#### RoleStore

```rust
#[async_trait]
pub trait RoleStore: Send + Sync {
    async fn create_role(&mut self, role: Role) -> Result<Role>;
    async fn get_role(&self, role_name: &str) -> Result<Option<Role>>;
    async fn update_role(&mut self, role: Role) -> Result<Role>;
    async fn delete_role(&mut self, role_name: &str) -> Result<()>;
    async fn list_roles(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Role>, bool, Option<String>)>;
    async fn tag_role(&mut self, role_name: &str, tags: Vec<Tag>) -> Result<()>;
    async fn list_role_tags(&self, role_name: &str) -> Result<Vec<Tag>>;
    async fn untag_role(&mut self, role_name: &str, tag_keys: Vec<String>) -> Result<()>;
}
```

#### AccessKeyStore

```rust
#[async_trait]
pub trait AccessKeyStore: Send + Sync {
    async fn create_access_key(&mut self, access_key: AccessKey) -> Result<AccessKey>;
    async fn get_access_key(&self, access_key_id: &str) -> Result<Option<AccessKey>>;
    async fn update_access_key(&mut self, access_key: AccessKey) -> Result<AccessKey>;
    async fn delete_access_key(&mut self, access_key_id: &str) -> Result<()>;
    async fn list_access_keys(&self, user_name: &str) -> Result<Vec<AccessKey>>;
}
```

#### StsStore Composite Trait

```rust
pub trait StsStore: SessionStore + IdentityStore + Send + Sync {}
```

#### SessionStore

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(&mut self, session: StsSession) -> Result<StsSession>;
    async fn get_session(&self, session_token: &str) -> Result<Option<StsSession>>;
    async fn delete_session(&mut self, session_token: &str) -> Result<()>;
    async fn list_sessions(&self, user_id: Option<&str>) -> Result<Vec<StsSession>>;
}
```

#### TenantStore

```rust
#[async_trait]
pub trait TenantStore: Send + Sync {
    async fn create_tenant(&mut self, tenant: Tenant) -> Result<Tenant>;
    async fn get_tenant(&self, tenant_id: &TenantId) -> Result<Option<Tenant>>;
    async fn update_tenant(&mut self, tenant: Tenant) -> Result<Tenant>;
    async fn delete_tenant(&mut self, tenant_id: &TenantId) -> Result<()>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>>;
    async fn list_child_tenants(&self, parent_id: &TenantId) -> Result<Vec<Tenant>>;
    async fn get_ancestors(&self, tenant_id: &TenantId) -> Result<Vec<Tenant>>;
    async fn get_descendants(&self, tenant_id: &TenantId) -> Result<Vec<TenantId>>;
    async fn get_effective_quotas(&self, tenant_id: &TenantId) -> Result<TenantQuotas>;
    async fn get_tenant_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage>;
}
```

### In-Memory Store Implementations

#### InMemoryWamiStore

```rust
pub struct InMemoryWamiStore {
    // Internal HashMap-based storage
    // Thread-safe with Arc<RwLock<T>>
}

impl InMemoryWamiStore {
    pub fn new() -> Self;
}

// Automatically implements all WamiStore sub-traits
```

#### InMemoryStsStore

```rust
pub struct InMemoryStsStore {
    // Internal HashMap-based storage
}

impl InMemoryStsStore {
    pub fn new() -> Self;
}

// Implements SessionStore and IdentityStore
```

#### InMemoryTenantStore

```rust
pub struct InMemoryTenantStore {
    // Internal HashMap-based storage
}

impl InMemoryTenantStore {
    pub fn new() -> Self;
}

// Implements TenantStore
```

---

## Provider Layer (provider::*)

### CloudProvider Trait

```rust
pub trait CloudProvider: Send + Sync {
    fn name(&self) -> &str;
    fn generate_arn(&self, resource_type: &str, resource_name: &str, account_id: &str) -> String;
    fn generate_id(&self, prefix: &str) -> String;
}
```

### Built-in Providers

#### AwsProvider

```rust
pub struct AwsProvider;

impl AwsProvider {
    pub fn new() -> Self;
}

// Generates ARNs like: arn:aws:iam::123456789012:user/alice
```

#### GcpProvider

```rust
pub struct GcpProvider;

impl GcpProvider {
    pub fn new() -> Self;
}

// Generates resource names in GCP format
```

#### AzureProvider

```rust
pub struct AzureProvider;

impl AzureProvider {
    pub fn new() -> Self;
}

// Generates resource IDs in Azure format
```

---

## Error Handling

### AmiError

```rust
pub enum AmiError {
    ValidationError(String),
    ResourceNotFound { resource: String },
    ResourceExists { resource: String },
    AccessDenied { message: String },
    InternalError(String),
    AwsSdk(aws_sdk_iam::Error),
    StsSdk(aws_sdk_sts::Error),
    SsoAdminSdk(aws_sdk_ssoadmin::Error),
}

impl std::error::Error for AmiError {}
impl std::fmt::Display for AmiError {}
```

---

## Common Types

### PaginationParams

```rust
pub struct PaginationParams {
    pub max_items: Option<usize>,
    pub marker: Option<String>,
}
```

### Tag

```rust
pub struct Tag {
    pub key: String,
    pub value: String,
}
```

### ProviderConfig

```rust
pub struct ProviderConfig {
    pub provider_name: String,
    pub account_id: String,
    pub native_arn: String,
    pub synced_at: DateTime<Utc>,
    pub tenant_id: Option<TenantId>,
}
```

---

## See Also

- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - Design and components
- **[Examples](EXAMPLES.md)** - Working code examples
- **[Store Implementation](STORE_IMPLEMENTATION.md)** - Create custom stores

---

For detailed examples and usage patterns, see the [Getting Started Guide](GETTING_STARTED.md).
