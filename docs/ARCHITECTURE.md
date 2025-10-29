# WAMI Architecture

Understanding WAMI's design principles and components.

## Overview

WAMI (Who Am I) is a **domain-driven**, cloud-agnostic Identity and Access Management library built in Rust. It separates business logic from storage, providing maximum flexibility while maintaining type safety and performance.

## Design Principles

1. **Pure Domain Logic** - Business rules without storage dependencies
2. **Storage Agnostic** - Works with any backend (memory, SQL, NoSQL)
3. **Multi-cloud** - Unified API across AWS, GCP, Azure
4. **Multi-tenant** - Built-in hierarchical tenant isolation
5. **Type Safe** - Rust's type system prevents common errors
6. **Async First** - Built on Tokio for high performance
7. **Testable** - Pure functions are easy to test in isolation

---

## High-Level Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                    Application Layer                          │
│                                                               │
│  Your application code using WAMI domain functions           │
│  and store traits                                            │
└────────────────────────────┬──────────────────────────────────┘
                             │
┌────────────────────────────┼──────────────────────────────────┐
│                    Domain Layer                               │
│                    (wami::*)                                  │
│                                                               │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Pure Functions (No Storage Dependencies)          │     │
│  │                                                     │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐       │     │
│  │  │ Identity │  │Credentials│  │ Policies │       │     │
│  │  │  • User  │  │ • AccessKey│  │ • Policy │       │     │
│  │  │  • Group │  │ • MFA     │  │ • Eval   │       │     │
│  │  │  • Role  │  │ • Login   │  └──────────┘       │     │
│  │  └──────────┘  └──────────┘                       │     │
│  │                                                     │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐       │     │
│  │  │   STS    │  │SSO Admin │  │  Tenant  │       │     │
│  │  │ • Session│  │ • PermSet│  │ • Hierarchy│      │     │
│  │  │ • Assume │  │ • Assign │  │ • Quotas │       │     │
│  │  └──────────┘  └──────────┘  └──────────┘       │     │
│  │                                                     │     │
│  │  Each module contains:                            │     │
│  │  • model.rs    - Domain entities                 │     │
│  │  • builder.rs  - Construction functions          │     │
│  │  • operations.rs - Business logic & validation   │     │
│  └────────────────────────────────────────────────────┘     │
└────────────────────────────┬──────────────────────────────────┘
                             │
┌────────────────────────────┼──────────────────────────────────┐
│                    Storage Layer                              │
│                    (store::*)                                 │
│                                                               │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Storage Traits (Persistence Interfaces)           │     │
│  │                                                     │     │
│  │  ┌─────────────────────────────────────┐          │     │
│  │  │  WamiStore (Composite Trait)        │          │     │
│  │  │  ├─ UserStore                       │          │     │
│  │  │  ├─ GroupStore                      │          │     │
│  │  │  ├─ RoleStore                       │          │     │
│  │  │  ├─ AccessKeyStore                  │          │     │
│  │  │  ├─ MfaDeviceStore                  │          │     │
│  │  │  ├─ PolicyStore                     │          │     │
│  │  │  └─ ... (12 sub-traits)             │          │     │
│  │  └─────────────────────────────────────┘          │     │
│  │                                                     │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐       │     │
│  │  │ StsStore │  │TenantStore│  │SsoAdminStore│    │     │
│  │  └──────────┘  └──────────┘  └──────────┘       │     │
│  └────────────────────────────────────────────────────┘     │
│                             │                                │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Storage Implementations                            │     │
│  │                                                     │     │
│  │  ┌───────────────────┐  ┌───────────────────┐    │     │
│  │  │InMemoryWamiStore  │  │ Your Custom Store │    │     │
│  │  │ (Built-in)        │  │ (SQL, NoSQL, etc) │    │     │
│  │  │                   │  │                   │    │     │
│  │  │ • HashMap-based   │  │ • PostgreSQL      │    │     │
│  │  │ • Thread-safe     │  │ • DynamoDB        │    │     │
│  │  │ • RwLock          │  │ • Redis           │    │     │
│  │  └───────────────────┘  └───────────────────┘    │     │
│  └────────────────────────────────────────────────────┘     │
└────────────────────────────┬──────────────────────────────────┘
                             │
┌────────────────────────────┼──────────────────────────────────┐
│                    Provider Layer                             │
│                                                               │
│  Cloud provider abstractions for ARN generation and          │
│  resource identification                                      │
│                                                               │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────────┐               │
│  │ AWS  │  │ GCP  │  │Azure │  │ Custom   │               │
│  └──────┘  └──────┘  └──────┘  └──────────┘               │
└───────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Domain Layer (wami::*)

**Purpose**: Pure business logic without storage dependencies

**Structure**:
```
wami/
├── identity/         # Users, groups, roles, service-linked roles
├── credentials/      # Access keys, MFA devices, login profiles, certificates
├── policies/         # IAM policies and policy evaluation
├── sts/             # Sessions, temporary credentials, identity
├── sso_admin/       # Permission sets, account assignments, applications
└── tenant/          # Multi-tenant models and quotas
```

**Each module contains**:
- `model.rs` - Domain entities (structs, enums)
- `builder.rs` - Pure construction functions
- `operations.rs` - Business logic and validation functions (optional)
- `requests.rs` - Request/response types (optional)

**Example - User Module**:
```rust
// wami/identity/user/model.rs
pub struct User {
    pub user_name: String,
    pub arn: String,
    pub user_id: String,
    pub path: Option<String>,
    pub created_at: DateTime<Utc>,
    // ... more fields
}

// wami/identity/user/builder.rs
pub fn build_user(
    user_name: String,
    path: Option<String>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> User {
    User {
        user_name: user_name.clone(),
        arn: provider.generate_arn("user", &user_name, account_id),
        user_id: generate_user_id(),
        path,
        created_at: Utc::now(),
        // ...
    }
}

// wami/identity/user/operations.rs (optional)
pub fn validate_user_name(name: &str) -> Result<()> {
    if name.len() > 64 {
        return Err(AmiError::ValidationError("User name too long".into()));
    }
    Ok(())
}
```

**Key Characteristics**:
- ✅ No `impl` blocks with storage dependencies
- ✅ Pure functions only
- ✅ Easy to test in isolation
- ✅ Can be used with any storage backend

---

### 2. Storage Layer (store::*)

**Purpose**: Define persistence contracts and provide implementations

**Structure**:
```
store/
├── traits/          # Storage trait definitions
│   ├── wami/       # WamiStore composite trait
│   │   ├── identity/
│   │   │   ├── user.rs          # UserStore trait
│   │   │   ├── group.rs         # GroupStore trait
│   │   │   └── role.rs          # RoleStore trait
│   │   ├── credentials/
│   │   ├── policies/
│   │   └── ...
│   ├── sts/        # STS store traits
│   ├── tenant/     # Tenant store traits
│   └── sso_admin/  # SSO store traits
│
└── memory/         # In-memory implementations
    ├── wami/       # InMemoryWamiStore
    ├── sts/        # InMemoryStsStore
    ├── tenant/     # InMemoryTenantStore
    └── sso_admin/  # InMemorySsoAdminStore
```

**Storage Trait Example**:
```rust
// store/traits/wami/identity/user.rs
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
}
```

**Composite Trait Pattern**:
```rust
// store/traits/wami/mod.rs
pub trait WamiStore: 
    UserStore + 
    GroupStore + 
    RoleStore + 
    AccessKeyStore + 
    PolicyStore + 
    // ... all sub-traits
    Send + Sync 
{}

// Automatic implementation for any type that implements all sub-traits
impl<T> WamiStore for T where
    T: UserStore + GroupStore + RoleStore + /* ... */ + Send + Sync
{}
```

**Benefits**:
- ✅ **Interface Segregation** - Clients only depend on what they need
- ✅ **Composability** - Combine traits as needed
- ✅ **Easy Testing** - Mock individual traits
- ✅ **Flexibility** - Implement only what you need

---

### 3. In-Memory Store

**Purpose**: Production-ready reference implementation

**Implementation**:
```rust
// store/memory/wami/mod.rs
#[derive(Debug, Clone)]
pub struct InMemoryWamiStore {
    users: Arc<RwLock<HashMap<String, User>>>,
    groups: Arc<RwLock<HashMap<String, Group>>>,
    roles: Arc<RwLock<HashMap<String, Role>>>,
    // ... more collections
}

impl InMemoryWamiStore {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            // ...
        }
    }
}
```

**Features**:
- Thread-safe with `Arc<RwLock<T>>`
- Suitable for testing and small deployments
- Full trait compliance
- Well-tested (256+ tests)

---

### 4. Provider Layer

**Purpose**: Cloud provider abstractions for resource identification

```rust
pub trait CloudProvider: Send + Sync {
    fn name(&self) -> &str;
    fn generate_arn(&self, resource_type: &str, resource_name: &str, account_id: &str) -> String;
    fn generate_id(&self, prefix: &str) -> String;
}

// Implementations
pub struct AwsProvider;
pub struct GcpProvider;
pub struct AzureProvider;
pub struct CustomProvider;
```

**Usage in builders**:
```rust
let user = user::builder::build_user(
    "alice".to_string(),
    None,
    &AwsProvider::new(),  // Provider determines ARN format
    "123456789012"
);
// user.arn = "arn:aws:iam::123456789012:user/alice"
```

---

## Data Flow

### Creating a Resource

```
1. Application builds domain model
   ↓
   user_builder::build_user(...) 
   → Returns User struct

2. Application persists to store
   ↓
   store.create_user(user)
   → UserStore::create_user()
   → Returns Result<User>
```

### Retrieving a Resource

```
1. Application queries store
   ↓
   store.get_user("alice")
   → UserStore::get_user()
   → Returns Result<Option<User>>

2. Application uses domain model
   ↓
   Operations on User struct
```

---

## Multi-Tenant Architecture

### Tenant Hierarchy

```
┌─────────────────────────────────────┐
│         Root Tenant                  │
│         (acme-corp)                  │
└──────────┬───────────────────────────┘
           │
    ┌──────┴──────┐
    │             │
┌───┴────┐   ┌───┴────┐
│ Eng    │   │ Sales  │
│ Dept   │   │ Dept   │
└───┬────┘   └────────┘
    │
┌───┴────┐
│ Team A │
└────────┘
```

### Tenant Isolation

Resources are isolated per tenant:
- Each resource has optional `tenant_id: Option<TenantId>`
- Store implementations enforce isolation
- Hierarchical permissions via ancestor/descendant queries

### Tenant Store Operations

```rust
#[async_trait]
pub trait TenantStore: Send + Sync {
    async fn create_tenant(&mut self, tenant: Tenant) -> Result<Tenant>;
    async fn get_tenant(&self, tenant_id: &TenantId) -> Result<Option<Tenant>>;
    async fn list_child_tenants(&self, parent_id: &TenantId) -> Result<Vec<Tenant>>;
    async fn get_ancestors(&self, tenant_id: &TenantId) -> Result<Vec<Tenant>>;
    async fn get_descendants(&self, tenant_id: &TenantId) -> Result<Vec<TenantId>>;
    async fn get_effective_quotas(&self, tenant_id: &TenantId) -> Result<TenantQuotas>;
}
```

---

## Testing Strategy

### 1. Domain Layer Tests

Pure functions are easy to test:

```rust
#[test]
fn test_build_user() {
    let provider = AwsProvider::new();
    let user = user::builder::build_user(
        "alice".to_string(),
        Some("/".to_string()),
        &provider,
        "123456789012"
    );
    
    assert_eq!(user.user_name, "alice");
    assert_eq!(user.arn, "arn:aws:iam::123456789012:user/alice");
}
```

### 2. Store Implementation Tests

Test CRUD operations, pagination, isolation:

```rust
#[tokio::test]
async fn test_user_store_crud() {
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    
    let user = user::builder::build_user("alice".into(), None, &provider, "123");
    
    // Create
    store.create_user(user.clone()).await.unwrap();
    
    // Read
    let retrieved = store.get_user("alice").await.unwrap();
    assert!(retrieved.is_some());
    
    // Update
    let mut updated = retrieved.unwrap();
    updated.path = Some("/new/".into());
    store.update_user(updated).await.unwrap();
    
    // Delete
    store.delete_user("alice").await.unwrap();
    assert!(store.get_user("alice").await.unwrap().is_none());
}
```

### 3. Integration Tests

Test complete workflows across multiple stores.

---

## Extending WAMI

### Adding a Custom Store

1. **Implement storage traits**:

```rust
pub struct PostgresWamiStore {
    pool: PgPool,
}

#[async_trait]
impl UserStore for PostgresWamiStore {
    async fn create_user(&mut self, user: User) -> Result<User> {
        sqlx::query!(
            "INSERT INTO users (user_name, arn, ...) VALUES ($1, $2, ...)",
            user.user_name, user.arn, ...
        )
        .execute(&self.pool)
        .await?;
        Ok(user)
    }
    
    async fn get_user(&self, user_name: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE user_name = $1",
            user_name
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }
    // ... implement other methods
}

// Implement all required sub-traits...
// PostgresWamiStore automatically implements WamiStore!
```

2. **Use it in your application**:

```rust
let pool = PgPoolOptions::new().connect("postgres://...").await?;
let mut store = PostgresWamiStore::new(pool);

// Same API as in-memory store!
let user = user::builder::build_user(...);
store.create_user(user).await?;
```

---

## Performance Considerations

### In-Memory Store

- **Reads**: O(1) with HashMap
- **Writes**: O(1) with locking overhead
- **List**: O(n) iteration with filtering
- **Thread-safe**: `Arc<RwLock<T>>` allows concurrent reads

### Custom Stores

- SQL stores: Optimize with indexes, prepared statements
- NoSQL stores: Optimize with compound keys, indexes
- Caching: Add caching layer for frequently accessed resources

---

## Comparison with Other Architectures

### ❌ Traditional "Fat Client" Pattern

```
┌──────────────────────────────┐
│   IamClient                  │
│   ├─ store: Arc<S>           │
│   ├─ create_user() {         │
│   │    let user = User {...} │
│   │    self.store.save(user) │
│   │  }                        │
│   └─ ...                      │
└──────────────────────────────┘
```

**Problems**:
- Storage tightly coupled to client
- Hard to test business logic in isolation
- Difficult to reuse domain logic
- Cannot compose operations easily

### ✅ WAMI's Pure Function Pattern

```
┌──────────────────────────────┐
│   Pure Functions             │
│   build_user(...) -> User    │
└──────────────────────────────┘
            ↓
┌──────────────────────────────┐
│   Store Traits               │
│   create_user(user)          │
└──────────────────────────────┘
```

**Benefits**:
- ✅ Separation of concerns
- ✅ Easy to test
- ✅ Reusable domain logic
- ✅ Flexible composition
- ✅ Storage-agnostic

---

## Summary

WAMI's architecture provides:

- **Clean separation** between domain logic and storage
- **Flexibility** to use any storage backend
- **Testability** with pure functions
- **Type safety** with Rust's type system
- **Performance** with async operations
- **Extensibility** through trait composition

The result is a library that's easy to use, test, and extend while maintaining high performance and type safety.
