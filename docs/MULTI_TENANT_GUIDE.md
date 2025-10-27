# Multi-Tenant Architecture

## Overview

WAMI now supports hierarchical multi-tenancy, allowing you to create isolated tenant environments with quota management, permission-based access control, and resource isolation.

## Features

### ✅ Implemented

- **Hierarchical Tenant Structure**
  - Unlimited depth with configurable constraints
  - Parent-child relationships with automatic ancestry tracking
  - Tenant IDs in format: `root` or `root/child` or `root/child/grandchild`

- **Quota Management**
  - Per-tenant resource quotas (users, roles, policies, groups, sub-tenants)
  - Quota inheritance from parent tenants
  - Override quotas for specific tenants
  - Automatic validation (child quotas cannot exceed parent)

- **Permission-Based Access Control**
  - Tenant admin principals (user ARNs)
  - Hierarchical permissions (parent admins can access child tenants)
  - Action-based authorization (Read, Update, Delete, CreateSubTenant, etc.)

- **Tenant Store & Client**
  - `TenantStore` trait for pluggable storage backends
  - `InMemoryTenantStore` implementation
  - `TenantClient` for tenant management operations
  - Full CRUD operations plus hierarchy queries

- **Tenant-Aware Resource Paths**
  - Helper functions for generating tenant-scoped paths
  - Format: `/tenants/{tenant_id}/` (e.g., `/tenants/acme/engineering/`)
  - ARN example: `arn:aws:iam::123456789012:user/tenants/acme/engineering/alice`
  - Extract tenant ID from paths

- **Comprehensive Testing**
  - 16 unit tests covering all major functionality
  - Model tests (TenantId, hierarchy, quotas)
  - Store tests (CRUD, hierarchy queries)
  - Client tests (operations, permissions, quota enforcement)

### 🔄 Optional Enhancements (Future)

- **IAM Resource Integration**
  - Add `tenant_id` field to IAM resources (User, Role, Policy, etc.)
  - Tenant-scoped IAM operations
  - Cross-tenant resource sharing

- **Advanced Features**
  - Tenant-scoped API keys
  - Audit logging per tenant
  - Usage metering and billing
  - Tenant data export/import
  - Tenant suspension/reactivation

## Architecture

### Core Components

```
src/tenant/
├── mod.rs              # Module declaration and exports
├── model.rs            # TenantId, Tenant, TenantQuotas, TenantStatus, etc.
├── client.rs           # TenantClient with operations
└── tests.rs           # Comprehensive test suite
```

### Data Model

#### TenantId
```rust
pub struct TenantId(String);

// Methods
pub fn root(name: &str) -> Self;
pub fn child(&self, name: &str) -> Self;
pub fn parent(&self) -> Option<Self>;
pub fn depth(&self) -> usize;
pub fn ancestors(&self) -> Vec<TenantId>;
pub fn is_descendant_of(&self, other: &TenantId) -> bool;
```

#### Tenant
```rust
pub struct Tenant {
    pub id: TenantId,
    pub parent_id: Option<TenantId>,
    pub name: String,
    pub organization: Option<String>,
    pub tenant_type: TenantType,
    pub provider_accounts: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub status: TenantStatus,
    pub quotas: TenantQuotas,
    pub quota_mode: QuotaMode,
    pub max_child_depth: usize,
    pub can_create_sub_tenants: bool,
    pub admin_principals: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub billing_info: Option<BillingInfo>,
}
```

#### TenantQuotas
```rust
pub struct TenantQuotas {
    pub max_users: usize,
    pub max_roles: usize,
    pub max_policies: usize,
    pub max_groups: usize,
    pub max_access_keys: usize,
    pub max_sub_tenants: usize,
    pub api_rate_limit: usize,
}
```

### TenantClient Operations

```rust
// Create tenants
async fn create_root_tenant(&mut self, request: CreateRootTenantRequest) -> Result<Tenant>;
async fn create_sub_tenant(&mut self, parent_id: &TenantId, request: CreateSubTenantRequest) -> Result<Tenant>;

// Query tenants
async fn get_tenant(&mut self, tenant_id: &TenantId) -> Result<Tenant>;
async fn list_child_tenants(&mut self, parent_id: &TenantId) -> Result<Vec<Tenant>>;
async fn get_tenant_usage(&mut self, tenant_id: &TenantId) -> Result<TenantUsage>;

// Delete tenants
async fn delete_tenant(&mut self, tenant_id: &TenantId, cascade: bool) -> Result<()>;
```

## Usage Examples

### Basic Setup

```rust
use wami::store::memory::InMemoryStore;
use wami::tenant::TenantClient;

let store = InMemoryStore::new();
let mut tenant_client = TenantClient::new(store, "admin@example.com".to_string());
```

### Create Root Tenant

```rust
use wami::tenant::client::CreateRootTenantRequest;
use wami::tenant::TenantQuotas;

let request = CreateRootTenantRequest {
    name: "acme".to_string(),
    organization: Some("Acme Corp".to_string()),
    provider_accounts: HashMap::new(),
    quotas: Some(TenantQuotas::default()),
    max_child_depth: Some(5),
    admin_principals: vec!["admin@acme.com".to_string()],
    metadata: HashMap::new(),
    billing_info: None,
};

let tenant = tenant_client.create_root_tenant(request).await?;
```

### Create Sub-Tenant

```rust
use wami::tenant::client::CreateSubTenantRequest;
use wami::tenant::{TenantId, TenantType};

let parent_id = TenantId::root("acme");

let request = CreateSubTenantRequest {
    name: "engineering".to_string(),
    organization: None,
    tenant_type: TenantType::Department,
    provider_accounts: None,
    quotas: None, // Inherit from parent
    admin_principals: vec!["eng-admin@acme.com".to_string()],
    metadata: None,
    billing_info: None,
};

let child = tenant_client.create_sub_tenant(&parent_id, request).await?;
```

### Working with Hierarchy

```rust
use wami::tenant::TenantId;

let root = TenantId::root("acme");
let child = root.child("engineering");
let grandchild = child.child("frontend");

// Check hierarchy
assert_eq!(grandchild.depth(), 2);
assert!(grandchild.is_descendant_of(&root));
assert!(grandchild.is_descendant_of(&child));

// Get parent
assert_eq!(grandchild.parent(), Some(child.clone()));

// Get all ancestors
let ancestors = grandchild.ancestors();
// ["acme", "acme/engineering", "acme/engineering/frontend"]
```

### Tenant-Aware Resource Paths

```rust
use wami::provider::CloudProvider;

// Generate tenant-aware path
let path = <dyn CloudProvider>::tenant_aware_path(
    Some("acme/engineering"),
    "/"
);
// Result: "/tenants/acme/engineering/"

// Generate ARN with tenant path
let provider = AwsProvider::default();
let arn = provider.generate_resource_identifier(
    ResourceType::User,
    "123456789012",
    "/tenants/acme/engineering/",
    "alice"
);
// Result: "arn:aws:iam::123456789012:user/tenants/acme/engineering/alice"

// Extract tenant from path
let tenant = <dyn CloudProvider>::extract_tenant_from_path(
    "/tenants/acme/engineering/"
);
// Result: Some("acme/engineering")
```

### Quota Enforcement

```rust
// Quotas are automatically enforced
let parent_quotas = TenantQuotas {
    max_users: 100,
    max_roles: 50,
    ..Default::default()
};

let child_quotas = TenantQuotas {
    max_users: 200, // ERROR: Exceeds parent!
    ..Default::default()
};

// Validation happens during sub-tenant creation
let result = tenant_client.create_sub_tenant(&parent_id, request).await;
// Returns: Err(InvalidParameter { message: "max_users exceeds parent limit" })
```

## Integration Patterns

### Single-Tenant Mode (Backward Compatible)

```rust
// Without tenant context - works as before
let mut iam_client = IamClient::new(store);
let user = iam_client.create_user(CreateUserRequest {
    user_name: "alice".to_string(),
    path: Some("/".to_string()),
    ..Default::default()
}).await?;
```

### Multi-Tenant Mode (New)

```rust
// With tenant context - use tenant-aware paths
let tenant_id = TenantId::new("acme/engineering");
let path = <dyn CloudProvider>::tenant_aware_path(
    Some(tenant_id.as_str()),
    "/"
);

let mut iam_client = IamClient::new(store);
let user = iam_client.create_user(CreateUserRequest {
    user_name: "alice".to_string(),
    path: Some(path), // "/tenants/acme/engineering/"
    ..Default::default()
}).await?;

// User ARN will be:
// arn:aws:iam::123456789012:user/tenants/acme/engineering/alice
```

## Testing

Run tenant tests:
```bash
cargo test tenant::
```

Run all tests:
```bash
cargo test
```

Run the example:
```bash
cargo run --example multi_tenant
```

## Performance Considerations

### Current Implementation (In-Memory)

- **Read operations**: O(1) for direct lookups, O(n) for hierarchy queries
- **Write operations**: O(1) for CRUD operations
- **Memory usage**: Linear with number of tenants

### Future Optimizations

- Database backend for persistent storage
- Caching layer for frequently accessed tenants
- Indexed queries for hierarchy traversal
- Pagination for large tenant lists

## Security Considerations

### Permission Model

1. **Tenant Admins**: Users listed in `admin_principals` can manage the tenant
2. **Hierarchical Permissions**: Parent tenant admins can access child tenants
3. **Action-Based**: Specific actions (Read, Update, Delete, CreateSubTenant, etc.)

### Best Practices

- Always verify tenant context before operations
- Use tenant-aware paths for resource isolation
- Implement audit logging for tenant operations
- Regular quota monitoring and alerting
- Secure admin principal management

## Migration Guide

### From Single-Tenant to Multi-Tenant

1. **Phase 1**: Deploy multi-tenant code (backward compatible)
   - No changes needed to existing code
   - All resources continue to work in "default" tenant

2. **Phase 2**: Create tenant structure
   - Create root tenant for organization
   - Create sub-tenants for departments/teams

3. **Phase 3**: Migrate resources (optional)
   - Use tenant-aware paths for new resources
   - Gradually migrate existing resources if needed

## API Reference

See the full API documentation:
```bash
cargo doc --no-deps --open
```

## Examples

- `examples/multi_tenant.rs` - Comprehensive multi-tenant demo
- `src/tenant/tests.rs` - Test suite with usage examples

## Future Roadmap

- [ ] Add `tenant_id` field to all IAM resources
- [ ] Tenant-scoped IAM client operations
- [ ] Cross-tenant resource sharing
- [ ] Tenant-scoped API keys
- [ ] Usage metering and billing integration
- [ ] Tenant data export/import
- [ ] Tenant audit logs
- [ ] Database storage backend
- [ ] GraphQL API for tenant management
- [ ] Admin UI for tenant management

