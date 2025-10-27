# ARN-Centric Architecture Migration Guide

## 🎯 Overview

This guide documents the migration to ARN-centric architecture where:
- **ARN is the primary identifier** for all resources
- **Unified Store** with `get(arn)`, `query(pattern)` methods
- **Multi-provider support** via `providers: Vec<ProviderInfo>`
- **Opaque WAMI ARNs** for security: `arn:wami:iam:tenant-<hash>:user/alice`

## ✅ What's Done

### Phase 1: Infrastructure ✅
- [x] `WamiArnBuilder` - Generates opaque WAMI ARNs with hash
- [x] `ParsedArn` - Parses and validates ARN strings
- [x] `ProviderInfo` - Tracks native cloud provider info
- [x] `Resource` enum - Unified type for all resources

### Phase 2: In Progress 🔄
- [ ] Add `arn: String` field to all models
- [ ] Add `providers: Vec<ProviderInfo>` field to all models
- [ ] Update builders to generate WAMI ARN
- [ ] Implement unified `InMemoryStore`
- [ ] Update clients to use `store.get(arn)`

## 📝 Required Changes Per Model

Each model needs two new fields:

```rust
pub struct User {
    // NEW: WAMI ARN (primary identifier)
    pub arn: String,  // "arn:wami:iam:tenant-a1b2:user/alice"
    
    // KEEP: Original fields
    pub user_name: String,
    pub user_id: String,  // Will become optional/deprecated
    
    // NEW: Multi-provider support
    pub providers: Vec<ProviderInfo>,
    
    // ... rest of fields
}
```

### Models to Migrate

#### IAM Resources
1. ✅ `User` (`src/iam/user/model.rs`)
2. ✅ `Role` (`src/iam/role/model.rs`)
3. ✅ `Policy` (`src/iam/policy/model.rs`)
4. ✅ `Group` (`src/iam/group/model.rs`)
5. ⏳ `AccessKey` (`src/iam/access_key/model.rs`)
6. ⏳ `MfaDevice` (`src/iam/mfa_device/model.rs`)
7. ⏳ `LoginProfile` (`src/iam/login_profile/model.rs`)
8. ⏳ `ServerCertificate` (`src/iam/server_certificate/model.rs`)
9. ⏳ `ServiceSpecificCredential` (`src/iam/service_credential/model.rs`)
10. ⏳ `SigningCertificate` (`src/iam/signing_certificate/model.rs`)

#### STS Resources
11. ⏳ `StsSession` (`src/sts/session/model.rs`)
12. ⏳ `Credentials` (`src/sts/credentials/model.rs`)

#### Tenant Resources
13. ⏳ `Tenant` (`src/tenant/model.rs`)

## 🔨 Builder Changes

Each builder needs to generate WAMI ARN:

```rust
// Before
pub fn build_user(..., provider: &dyn CloudProvider, account_id: &str) -> User {
    let arn = provider.generate_resource_identifier(...);  // Native ARN
    User { arn, ... }
}

// After
pub fn build_user(..., provider: &dyn CloudProvider, account_id: &str) -> User {
    let arn_builder = WamiArnBuilder::new();
    let arn = arn_builder.build_arn("iam", account_id, "user", path, name);
    
    let native_arn = provider.generate_resource_identifier(...);
    let provider_info = ProviderInfo::new(
        provider.name(),
        native_arn,
        Some(user_id.clone()),
        account_id,
    );
    
    User { 
        arn,                           // WAMI ARN
        providers: vec![provider_info], // Native provider info
        ...
    }
}
```

## 🗄️ Store Changes

### Old API (per-resource methods)
```rust
pub trait IamStore {
    async fn get_user(&mut self, user_name: &str) -> Result<Option<User>>;
    async fn get_role(&mut self, role_name: &str) -> Result<Option<Role>>;
    // ... 20+ methods
}
```

### New API (unified methods)
```rust
pub trait Store {
    async fn get(&mut self, arn: &str) -> Result<Option<Resource>>;
    async fn query(&mut self, pattern: &str) -> Result<Vec<Resource>>;
    async fn put(&mut self, resource: Resource) -> Result<()>;
    async fn delete(&mut self, arn: &str) -> Result<()>;
}
```

## 📊 Migration Strategy

### Option 1: Big Bang (Current Approach)
- Migrate all models at once
- Update all builders
- Replace store implementation
- **Pros**: Clean break, consistent
- **Cons**: Large PR, high risk

### Option 2: Gradual Migration (Recommended)
1. Add `arn` and `providers` fields to models (keep old fields)
2. Update builders to set both old and new fields
3. Add new `get_by_arn()` methods alongside old methods
4. Gradually migrate clients to use ARN
5. Deprecate old fields/methods
6. Remove deprecated code in v2.0

## 🚀 Quick Start (For Testing)

```rust
use wami::provider::arn_builder::WamiArnBuilder;
use wami::provider::provider_info::ProviderInfo;

// Generate WAMI ARN
let builder = WamiArnBuilder::new();
let arn = builder.build_arn("iam", "123456789012", "user", "/", "alice");
// Result: arn:wami:iam:tenant-a1b2c3d4:user/alice

// Create provider info
let provider_info = ProviderInfo::new(
    "aws",
    "arn:aws:iam::123456789012:user/alice",
    Some("AIDACKCEVSQ6C2EXAMPLE".to_string()),
    "123456789012",
);

// Parse ARN
let parsed = ParsedArn::from_arn(&arn).unwrap();
assert_eq!(parsed.service, "iam");
assert_eq!(parsed.resource_type, "user");

// Pattern matching
let matches = parsed.matches_pattern("arn:wami:iam:tenant-*:user/*");
assert!(matches);
```

## 🔐 Security Benefits

### Before
```
arn:aws:iam::123456789012:user/tenants/acme/engineering/alice
                └─────────┘      └────────────────────────┘
             Account ID exposed  Tenant hierarchy exposed
```

### After
```
arn:wami:iam:tenant-a1b2c3d4:user/t/a/e/alice
                └──────────┘         └────┘
              Opaque hash          Obfuscated path
```

**Lookup**: `store.get(arn)` → Database → Real account ID

## 📚 Next Steps

1. **For Contributors**: See Phase 2 checklist above
2. **For Users**: API remains backward compatible during migration
3. **For Testing**: Use `WamiArnBuilder` directly (already works!)

## ❓ Questions

- **Q**: Do we lose native ARNs?
  - **A**: No! They're in `resource.providers[].native_arn`

- **Q**: Can we query by tenant?
  - **A**: Yes! `store.query("arn:wami:*:tenant-xyz:*")`

- **Q**: Is this a breaking change?
  - **A**: Can be gradual with dual-field approach

---

Last Updated: 2025-10-27
Status: Phase 1 Complete, Phase 2 In Progress

