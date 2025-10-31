# WAMI ARN Migration Guide

## Overview

This guide documents the migration from string-based ARNs to the structured `WamiArn` type with region support. This is a **breaking change** that provides better type safety, region support, and multi-cloud capabilities.

## What Changed

### 1. ARN Format Enhanced with Region

**Old Format (still supported for parsing):**
```
arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:user/77557755
```

**New Format:**
```
arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:us-east-1:user/77557755
                                                        ^^^^^^^^^^^
                                                        Region added
```

**Global Resources:**
```
arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:global:user/77557755
```

### 2. Resource Models Migration

**Before:**
```rust
pub struct User {
    pub user_name: String,
    pub wami_arn: String,  // String ARN
    // ... other fields
}
```

**After:**
```rust
pub struct User {
    pub user_name: String,
    pub wami_arn: WamiArn,  // Structured ARN
    // ... other fields
}
```

### 3. CloudMapping Enhanced

**Before:**
```rust
pub struct CloudMapping {
    pub provider: String,
    pub account_id: String,
}
```

**After:**
```rust
pub struct CloudMapping {
    pub provider: String,
    pub account_id: String,
    pub region: Option<String>,  // NEW
}
```

## Completed Migration Steps

### ✅ Phase 1: Core ARN Infrastructure

1. **ARN Types with Region Support** (`src/arn/types.rs`)
   - Added `region: Option<String>` to `CloudMapping`
   - Added helper methods: `with_region()`, `is_regional()`, `region_or_global()`
   - Updated `WamiArn::prefix()` to include region
   - Added comprehensive tests

2. **Parser with Region Support** (`src/arn/parser.rs`)
   - Supports new format: `...provider:account:region:resource`
   - **Backward compatible**: Still parses legacy format without region
   - Handles "global" as None region
   - Added roundtrip tests

3. **Builder with Region Methods** (`src/arn/builder.rs`)
   - `.cloud_provider(provider, account)` - creates global resource
   - `.cloud_provider_with_region(provider, account, region)` - creates regional resource
   - `.region(region)` - sets region on existing cloud mapping
   - Comprehensive tests

4. **Transformers Updated** (`src/arn/transformer.rs`)
   - AWS transformer uses region in ARN format
   - GCP, Azure, Scaleway transformers updated
   - `ProviderArnInfo` includes `region: Option<String>`
   - All tests passing

5. **String Serialization** (`src/arn/types.rs`)
   - Custom `Serialize` implementation: WamiArn → string
   - Custom `Deserialize` implementation: string → WamiArn
   - **JSON Compatible**: Serializes as `"arn:wami:..."`
   - Roundtrip tested

## Pending Migration Steps

### 🔄 Phase 2: Resource Models Migration

**Identity Models** (High Priority):
- [ ] `src/wami/identity/user/model.rs` - User
- [ ] `src/wami/identity/role/model.rs` - Role
- [ ] `src/wami/identity/group/model.rs` - Group
- [ ] `src/wami/identity/identity_provider/model.rs` - IdentityProvider
- [ ] `src/wami/identity/service_linked_role/model.rs` - ServiceLinkedRole

**Credential Models**:
- [ ] `src/wami/credentials/access_key/model.rs` - AccessKey
- [ ] `src/wami/credentials/login_profile/model.rs` - LoginProfile
- [ ] `src/wami/credentials/mfa_device/model.rs` - MfaDevice
- [ ] `src/wami/credentials/server_certificate/model.rs` - ServerCertificate
- [ ] `src/wami/credentials/service_credential/model.rs` - ServiceSpecificCredential
- [ ] `src/wami/credentials/signing_certificate/model.rs` - SigningCertificate

**Policy Models**:
- [ ] `src/wami/policies/policy/model.rs` - Policy

**STS Models**:
- [ ] `src/wami/sts/credentials/model.rs` - Credentials
- [ ] `src/wami/sts/session/model.rs` - StsSession

**SSO Admin Models**:
- [ ] `src/wami/sso_admin/instance/model.rs` - SsoInstance
- [ ] `src/wami/sso_admin/permission_set/model.rs` - PermissionSet
- [ ] `src/wami/sso_admin/account_assignment/model.rs` - AccountAssignment
- [ ] `src/wami/sso_admin/application/model.rs` - Application
- [ ] `src/wami/sso_admin/trusted_token_issuer/model.rs` - TrustedTokenIssuer

### 🔄 Phase 3: Builders Migration

**Identity Builders**:
- [ ] `src/wami/identity/user/builder.rs` - build_user()
- [ ] `src/wami/identity/role/builder.rs` - build_role()
- [ ] `src/wami/identity/group/builder.rs` - build_group()
- [ ] All credential builders
- [ ] Policy builders
- [ ] STS builders
- [ ] SSO Admin builders

### 🔄 Phase 4: Services Migration

Update all services that work with ARNs:
- [ ] `src/service/identity/user.rs` - UserService
- [ ] `src/service/identity/role.rs` - RoleService
- [ ] `src/service/identity/group.rs` - GroupService
- [ ] All credential services
- [ ] Policy services
- [ ] STS services
- [ ] SSO Admin services

### 🔄 Phase 5: Examples Migration

Update all 24+ examples:
- [ ] `examples/01_hello_wami.rs`
- [ ] `examples/02_basic_crud_operations.rs`
- [ ] ... (all examples)
- [ ] Create `examples/26_arn_migration.rs` showing migration patterns

### 🔄 Phase 6: Documentation

- [ ] Update `docs/ARN_SPECIFICATION.md` with region details
- [ ] Add region examples to all ARN documentation
- [ ] Update API docs for affected types

## Migration Patterns

### Pattern 1: Creating ARNs in Builders

**Before:**
```rust
fn build_user(name: String, path: Option<String>, provider: &impl CloudProvider, account_id: &str) -> User {
    let arn = provider.generate_user_arn(account_id, &name, path.as_deref());
    
    User {
        user_name: name,
        wami_arn: arn,  // String
        // ...
    }
}
```

**After:**
```rust
use wami::arn::{WamiArn, Service};

fn build_user(
    name: String,
    path: Option<String>,
    provider: &impl CloudProvider,
    account_id: &str,
    tenant_id: &str,
    instance_id: &str,
    user_id: &str,
) -> User {
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant(tenant_id)
        .wami_instance(instance_id)
        .cloud_provider(provider.name(), account_id)
        .resource("user", user_id)
        .build()
        .unwrap();
    
    User {
        user_name: name,
        wami_arn,  // WamiArn type
        // ...
    }
}
```

### Pattern 2: Creating Regional ARNs

```rust
let arn = WamiArn::builder()
    .service(Service::Iam)
    .tenant_hierarchy(vec!["acme", "engineering", "backend"])
    .wami_instance("prod-001")
    .cloud_provider_with_region("aws", "123456789012", "us-east-1")
    .resource("user", "AIDACKCEVSQ6C2EXAMPLE")
    .build()?;
```

### Pattern 3: Querying by Tenant and Region

```rust
// Get all resources in tenant t1/t2/t3
let tenant_prefix = "arn:wami:iam:t1/t2/t3:wami:999888777";

// Get all resources in tenant t1/t2/t3 in us-east-1
let regional_prefix = "arn:wami:iam:t1/t2/t3:wami:999888777:aws:123456:us-east-1";

for resource in all_resources {
    if resource.wami_arn.matches_prefix(tenant_prefix) {
        println!("Resource in tenant: {}", resource.wami_arn);
    }
}
```

### Pattern 4: Accessing ARN Components

**Before:**
```rust
// Had to parse string manually
if user.wami_arn.contains("aws") {
    // ...
}
```

**After:**
```rust
// Type-safe access
if user.wami_arn.is_cloud_synced() {
    println!("Provider: {}", user.wami_arn.provider().unwrap());
    println!("Tenant: {}", user.wami_arn.full_tenant_path());
    println!("Region: {}", user.wami_arn.cloud_mapping.as_ref().unwrap().region_or_global());
}
```

### Pattern 5: JSON Serialization

**JSON Output (Unchanged):**
```json
{
  "user_name": "alice",
  "wami_arn": "arn:wami:iam:t1:wami:999888777:aws:123456:us-east-1:user/AIDACK123"
}
```

The ARN is automatically serialized as a string, maintaining API compatibility.

### Pattern 6: Parsing ARNs

```rust
use std::str::FromStr;
use wami::arn::WamiArn;

// From string
let arn = WamiArn::from_str("arn:wami:iam:t1:wami:999888777:user/123")?;

// Or using parse_arn helper
let arn = wami::arn::parse_arn("arn:wami:iam:t1:wami:999888777:user/123")?;
```

## Breaking Changes

### 1. Type Changes

All resources now have `wami_arn: WamiArn` instead of `wami_arn: String`.

**Impact:**
- Code that accessed `.wami_arn` directly will need updates
- JSON deserialization still works (backward compatible)
- Comparison operations change from string to struct

### 2. Builder Signatures

Builders now require additional parameters:
- `tenant_id`: For multi-tenant support
- `instance_id`: For WAMI instance identification
- `resource_id`: Stable ID (not resource name)

### 3. Service Methods

Services that return resources now return `WamiArn` types.

## Backward Compatibility

### Parsing Legacy ARNs

The parser supports legacy format without region:
```rust
// Legacy format (without region) still works
let arn = WamiArn::from_str("arn:wami:iam:t1:wami:999888777:aws:123456:user/123")?;
assert_eq!(arn.cloud_mapping.unwrap().region, None);
```

### JSON Compatibility

JSON serialization remains string-based for compatibility:
```json
"wami_arn": "arn:wami:iam:t1:wami:999888777:user/123"
```

## Testing Strategy

### Unit Tests

Test ARN operations:
```rust
#[test]
fn test_arn_with_region() {
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant("t1")
        .wami_instance("999888777")
        .cloud_provider_with_region("aws", "123456", "us-east-1")
        .resource("user", "123")
        .build()
        .unwrap();
    
    assert_eq!(arn.cloud_mapping.as_ref().unwrap().region, Some("us-east-1".to_string()));
    assert!(arn.cloud_mapping.as_ref().unwrap().is_regional());
}
```

### Integration Tests

Test resource creation with ARNs:
```rust
#[tokio::test]
async fn test_create_user_with_arn() {
    let store = InMemoryWamiStore::default();
    let service = UserService::new(Arc::new(RwLock::new(store)));
    
    let request = CreateUserRequest {
        user_name: "alice".to_string(),
        // ...
    };
    
    let user = service.create_user(request).await.unwrap();
    assert_eq!(user.wami_arn.resource_type(), "user");
    assert_eq!(user.wami_arn.service, Service::Iam);
}
```

## Performance Considerations

### Minimal Overhead

- `WamiArn` is a lightweight struct (no heap allocations for fixed fields)
- Serialization to string is lazy (only when needed)
- Parsing is cached in the struct

### Memory Usage

- Slightly higher than plain strings due to structured data
- Trade-off for type safety and functionality

## Migration Checklist

### For Library Maintainers

- [x] Update ARN types with region support
- [x] Update parser for new format
- [x] Update builder with region methods
- [x] Update transformers
- [x] Add serialization support
- [ ] Migrate all resource models
- [ ] Update all builders
- [ ] Update all services
- [ ] Update all examples
- [ ] Update documentation
- [ ] Run full test suite
- [ ] Update CHANGELOG.md

### For Library Users

When upgrading to this version:

1. **Update type signatures:**
   ```rust
   // OLD
   fn process_user(arn: &str) { ... }
   
   // NEW
   fn process_user(arn: &WamiArn) { ... }
   ```

2. **Update ARN creation:**
   ```rust
   // OLD
   let arn = format!("arn:wami:iam:...:{}", resource_id);
   
   // NEW
   let arn = WamiArn::builder()
       .service(Service::Iam)
       .tenant(tenant_id)
       .wami_instance(instance_id)
       .resource("user", resource_id)
       .build()?;
   ```

3. **Update ARN access:**
   ```rust
   // OLD
   if arn.starts_with("arn:wami:iam") { ... }
   
   // NEW
   if arn.service == Service::Iam { ... }
   ```

4. **Add region where needed:**
   ```rust
   let arn = WamiArn::builder()
       // ... other fields
       .cloud_provider_with_region("aws", account_id, "us-east-1")
       .build()?;
   ```

## FAQ

### Q: Do I need to specify region for all resources?

**A:** No. Region is optional. For global services like IAM, omit the region or it will default to "global":
```rust
.cloud_provider("aws", account_id)  // Region will be None, displays as "global"
```

### Q: How do I query resources across all regions?

**A:** Use tenant-based prefix matching:
```rust
let tenant_prefix = format!("arn:wami:iam:{}:wami:{}", tenant_path, instance_id);
// This matches all resources in the tenant, regardless of region
```

### Q: Is the migration backward compatible?

**A:** Partially:
- ✅ ARN parsing supports legacy format
- ✅ JSON serialization is compatible
- ❌ Code using string ARNs must be updated
- ❌ Builder signatures changed (breaking)

### Q: How do I handle resources without tenant_id?

**A:** Use a default tenant like `"default"` or `"root"` for existing resources during migration.

### Q: Can I still use string ARNs temporarily?

**A:** You can convert:
```rust
// WamiArn to String
let arn_string = my_arn.to_string();

// String to WamiArn
let my_arn = WamiArn::from_str(&arn_string)?;
```

## Support

For migration assistance:
- Check examples in `examples/` directory
- See ARN specification in `docs/ARN_SPECIFICATION.md`
- Review tests in `src/arn/types.rs`, `src/arn/parser.rs`

## Version History

- **v0.11.0**: Initial ARN migration with region support
  - Added structured `WamiArn` type
  - Added region support to CloudMapping
  - Maintained backward compatibility for parsing
  - Breaking changes to resource models and builders

