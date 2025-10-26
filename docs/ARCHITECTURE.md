# WAMI Architecture

Understanding WAMI's design and components.

## Overview

WAMI (WebAssembly AWS IAM) is a pluggable, cloud-agnostic IAM library built in Rust. It provides AWS-compatible IAM operations while supporting multiple cloud providers and custom storage backends.

## Core Principles

1. **Cloud Agnostic** - Works with AWS, GCP, Azure, or custom providers
2. **Pluggable Storage** - Memory, database, or custom backends
3. **Multi-Tenant** - Built-in hierarchical tenant isolation
4. **Type Safe** - Rust's type system prevents common errors
5. **Async First** - Built on Tokio for high performance
6. **Extensible** - Easy to add new features and integrations

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│  (Your Code using IamClient, StsClient, SsoAdminClient)     │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┼────────────────────────────────────┐
│                   Client Layer                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │   IAM    │  │   STS    │  │ SSO Admin│  │  Tenant  │   │
│  │  Client  │  │  Client  │  │  Client  │  │  Client  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┼────────────────────────────────────┐
│                   Store Layer                                │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Store Trait (pluggable persistence)               │    │
│  ├────────────────────────────────────────────────────┤    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │    │
│  │  │ IamStore │  │ StsStore │  │TenantStore│  ...   │    │
│  │  └──────────┘  └──────────┘  └──────────┘        │    │
│  └────────────────────────────────────────────────────┘    │
│                         │                                    │
│  ┌──────────────────┐  │  ┌──────────────────┐            │
│  │  InMemoryStore   │──┘  │  Your Custom     │            │
│  │  (built-in)      │     │  Store (DB, etc) │            │
│  └──────────────────┘     └──────────────────┘            │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┼────────────────────────────────────┐
│                 Provider Layer                               │
│  ┌────────────────────────────────────────────────────┐    │
│  │  CloudProvider Trait (resource identifiers)        │    │
│  ├────────────────────────────────────────────────────┤    │
│  │  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────────┐      │    │
│  │  │ AWS  │  │ GCP  │  │Azure │  │ Custom   │      │    │
│  │  └──────┘  └──────┘  └──────┘  └──────────┘      │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Client Layer

**Purpose**: High-level API for IAM operations

**Components**:
- `IamClient<S>` - User, role, policy, group management
- `StsClient<S>` - Temporary credentials and sessions
- `SsoAdminClient<S>` - SSO permission sets and assignments
- `TenantClient` - Multi-tenant hierarchy management

**Responsibilities**:
- Validate request parameters
- Call appropriate store methods
- Handle errors
- Return structured responses

**Example**:
```rust
pub struct IamClient<S: Store> {
    store: S,
}

impl<S: Store> IamClient<S> {
    pub async fn create_user(&mut self, request: CreateUserRequest) -> Result<Response<User>> {
        // 1. Validate request
        validate_user_name(&request.user_name)?;
        
        // 2. Get store
        let iam_store = self.store.iam_store().await?;
        
        // 3. Build user
        let user = builder::build_user(request, &self.store)?;
        
        // 4. Store user
        let created = iam_store.create_user(user).await?;
        
        // 5. Return response
        Ok(Response::success(created))
    }
}
```

### 2. Store Layer

**Purpose**: Abstract persistence mechanism

**Key Traits**:
```rust
#[async_trait]
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

**Built-in Implementations**:
- `InMemoryStore` - HashMap-based (dev/testing)

**Custom Implementations** (examples):
- `PostgresStore` - PostgreSQL persistence
- `RedisStore` - Redis for sessions
- `DynamoDbStore` - AWS DynamoDB

### 3. Provider Layer

**Purpose**: Generate cloud-specific identifiers

**Key Trait**:
```rust
pub trait CloudProvider: Send + Sync {
    fn name(&self) -> &str;
    fn generate_resource_identifier(&self, type: &str, name: &str, path: Option<&str>) -> String;
    fn generate_account_id(&self) -> String;
    fn max_session_duration(&self) -> u32;
    fn max_users(&self) -> Option<u32>;
    fn max_roles(&self) -> Option<u32>;
    fn max_policies(&self) -> Option<u32>;
}
```

**Built-in Providers**:
- `AwsProvider` - AWS ARNs (`arn:aws:iam::123456789012:user/alice`)
- `GcpProvider` - GCP service accounts (`projects/proj/serviceAccounts/...`)
- `AzureProvider` - Azure ARM paths (`/subscriptions/.../users/alice`)

## Module Structure

```
src/
├── lib.rs                    # Public API exports
├── error.rs                  # Error types
├── types.rs                  # Shared types
│
├── iam/                      # IAM domain
│   ├── mod.rs               # IAM client
│   ├── user/                # Self-contained modules
│   │   ├── mod.rs
│   │   ├── model.rs         # User struct
│   │   ├── requests.rs      # CreateUserRequest, etc.
│   │   ├── operations.rs    # create_user, get_user, etc.
│   │   └── builder.rs       # User construction logic
│   ├── role/                # Similar structure
│   ├── policy/
│   ├── group/
│   └── ...
│
├── sts/                      # STS domain
│   ├── mod.rs               # STS client
│   ├── assume_role/
│   ├── session_token/
│   ├── federation/
│   ├── credentials/         # Shared credential logic
│   └── session/             # Session management
│
├── sso_admin/                # SSO Admin domain
│   ├── mod.rs
│   ├── permission_set/
│   └── assignment/
│
├── tenant/                   # Multi-tenant domain
│   ├── mod.rs
│   ├── model.rs             # Tenant, TenantId
│   ├── client.rs            # TenantClient
│   ├── hierarchy.rs         # Hierarchy utilities
│   └── store/               # Tenant persistence
│
├── store/                    # Storage abstraction
│   ├── mod.rs               # Store trait
│   ├── traits/              # Domain store traits
│   │   ├── iam.rs           # IamStore trait
│   │   ├── sts.rs           # StsStore trait
│   │   └── sso_admin.rs     # SsoAdminStore trait
│   └── memory/              # In-memory implementation
│       ├── mod.rs
│       ├── iam.rs           # InMemoryIamStore
│       ├── sts.rs           # InMemoryStsStore
│       ├── sso_admin.rs     # InMemorySsoAdminStore
│       ├── tenant.rs        # InMemoryTenantStore
│       └── unified.rs       # InMemoryStore (combines all)
│
└── provider/                 # Cloud provider abstraction
    ├── mod.rs               # CloudProvider trait
    ├── aws.rs               # AwsProvider
    ├── gcp.rs               # GcpProvider
    ├── azure.rs             # AzureProvider
    └── custom.rs            # CustomProvider
```

## Data Flow

### Example: Create User

```
1. Application
   ├─> IamClient::create_user(CreateUserRequest)
         │
2. Client Layer
   ├─> Validate request (user_name, path, etc.)
   ├─> Get CloudProvider from Store
   │     └─> provider.generate_resource_identifier("user", "alice", "/")
   │           → "arn:aws:iam::123456789012:user/alice"
   ├─> builder::build_user(request, provider)
   │     └─> Create User struct with ARN, timestamps, etc.
   │
3. Store Layer
   ├─> store.iam_store().await
   ├─> iam_store.create_user(user)
   │     └─> HashMap.insert(user_name, user)  [in-memory]
   │     └─> INSERT INTO users ... [database]
   │
4. Return
   └─> Response<User>
```

### Example: Assume Role

```
1. Application
   ├─> StsClient::assume_role(AssumeRoleRequest)
         │
2. Client Layer
   ├─> Validate request (role_arn, duration, etc.)
   ├─> Get role from IAM store
   ├─> Validate assume role policy
   ├─> Generate temporary credentials
   │     ├─> AccessKeyId: ASIA...
   │     ├─> SecretAccessKey: random
   │     ├─> SessionToken: random
   │     └─> Expiration: now + duration
   │
3. Store Layer
   ├─> store.sts_store().await
   ├─> sts_store.create_session(session)
   │     └─> Store session with expiration
   │
4. Return
   └─> Response<AssumeRoleResponse>
         └─> credentials: Credentials
```

## Self-Contained Module Pattern

WAMI uses a consistent module structure:

```
resource_name/
├── mod.rs          # Module declaration, exports
├── model.rs        # Data structures (User, Role, etc.)
├── requests.rs     # Request/Response types
├── operations.rs   # Client methods (create, get, list, delete)
└── builder.rs      # Construction logic
```

**Benefits**:
- **Encapsulation**: Each resource is self-contained
- **Discoverability**: Easy to find related code
- **Consistency**: Predictable structure across all resources
- **Maintainability**: Changes are localized

**Example**: `src/iam/user/`
```rust
// mod.rs
pub mod model;
pub mod requests;
pub mod operations;
pub mod builder;

pub use model::*;
pub use requests::*;

// model.rs
pub struct User {
    pub user_name: String,
    pub arn: String,
    // ...
}

// requests.rs
pub struct CreateUserRequest {
    pub user_name: String,
    // ...
}

pub struct CreateUserResponse {
    pub user: User,
}

// operations.rs
impl<S: Store> IamClient<S> {
    pub async fn create_user(&mut self, request: CreateUserRequest) -> Result<Response<User>> {
        // Implementation
    }
}

// builder.rs
pub fn build_user(request: CreateUserRequest, provider: &dyn CloudProvider) -> Result<User> {
    // Construction logic
}
```

## Multi-Tenancy

### Tenant Hierarchy

```
Root Tenant (Organization)
├── Department Tenant
│   ├── Team Tenant
│   └── Team Tenant
└── Department Tenant
    └── Team Tenant
```

### Tenant ID Format

```
{root}                    # Root tenant
{root}/{dept}             # Department
{root}/{dept}/{team}      # Team
```

### Tenant-Aware Resources

```rust
pub struct User {
    pub user_name: String,
    pub arn: String,
    pub tenant_id: Option<TenantId>,  // Tenant isolation
    // ...
}
```

### Tenant-Aware ARNs

```
Without tenant:
arn:aws:iam::123456789012:user/alice

With tenant:
arn:aws:iam::123456789012:user/tenants/acme/engineering/alice
```

## Error Handling

### Error Types

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

### Error Propagation

```rust
// Client validates and returns detailed errors
pub async fn create_user(&mut self, request: CreateUserRequest) -> Result<Response<User>> {
    if request.user_name.is_empty() {
        return Err(AmiError::InvalidParameter {
            message: "User name cannot be empty".to_string(),
        });
    }
    
    // Store propagates errors
    let iam_store = self.store.iam_store().await?;
    let user = iam_store.create_user(user).await?;
    
    Ok(Response::success(user))
}
```

## Performance Considerations

### 1. Async/Await

All I/O operations are async for high concurrency:

```rust
#[async_trait]
pub trait IamStore {
    async fn create_user(&mut self, user: User) -> Result<User>;
    async fn get_user(&self, user_name: &str) -> Result<Option<User>>;
}
```

### 2. Connection Pooling

For database stores:

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(database_url).await?;
```

### 3. Caching

Layer caching for frequently accessed data:

```rust
pub struct CachedStore<S: Store> {
    store: S,
    cache: Cache<String, User>,
}
```

### 4. Batch Operations

Support batch operations where possible:

```rust
async fn create_users(&mut self, users: Vec<User>) -> Result<Vec<User>>;
```

## Testing Strategy

### Unit Tests

Test individual components:

```rust
#[tokio::test]
async fn test_create_user() {
    let store = create_memory_store();
    let mut iam = MemoryIamClient::new(store);
    
    let result = iam.create_user(request).await;
    assert!(result.is_ok());
}
```

### Integration Tests

Test complete workflows:

```rust
#[tokio::test]
async fn test_user_workflow() {
    // Create user
    // Create access keys
    // Attach policy
    // Verify permissions
}
```

### Provider Tests

Test all providers consistently:

```rust
async fn test_with_provider(provider: Arc<dyn CloudProvider>) {
    // Same test, different provider
}
```

## Extension Points

### 1. Custom Store

Implement `Store` trait for your persistence layer.

### 2. Custom Provider

Implement `CloudProvider` trait for your cloud.

### 3. Custom Validation

Add middleware or validation layers.

### 4. Custom Policies

Extend policy evaluation logic.

## Next Steps

- **[Getting Started](GETTING_STARTED.md)** - Quick start
- **[Store Implementation](STORE_IMPLEMENTATION.md)** - Custom storage
- **[Multicloud Providers](MULTICLOUD_PROVIDERS.md)** - Cloud abstraction
- **[Examples](EXAMPLES.md)** - Code samples

## Support

Questions? Open an issue on [GitHub](https://github.com/lsh0x/wami/issues).

