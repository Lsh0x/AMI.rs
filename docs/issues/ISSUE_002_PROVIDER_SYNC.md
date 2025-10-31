# Issue #002: Implement Cloud Provider Synchronization

**Status**: 🔴 Open  
**Priority**: High  
**Type**: Feature Enhancement  
**Assignee**: TBD  
**Created**: 2025-01-27  
**Labels**: `enhancement`, `provider-sync`, `multi-cloud`, `aws-integration`, `gcp-integration`, `azure-integration`

## Summary

Implement automatic synchronization of WAMI resources (users, roles, groups, policies) to actual cloud providers (AWS, GCP, Azure) when they are created, updated, or deleted. This includes tracking sync state per provider and handling failures gracefully with retry mechanisms.

## Current State

- ✅ Resources are created in WAMI local storage only
- ✅ `ProviderConfig` tracks provider metadata (`synced_at`, `native_arn`, etc.)
- ✅ `User.providers` field exists but is always empty (`Vec::new()`) on creation
- ✅ Provider system generates provider-specific ARNs and IDs
- ❌ No actual API calls to cloud providers
- ❌ No sync state tracking (pending, syncing, synced, failed)
- ❌ No retry logic for failed syncs
- ❌ No configuration for which providers to sync to

## Problem Statement

Currently, when you create a user in WAMI:
1. ✅ User is stored in WAMI's local storage
2. ✅ User gets WAMI ARN and provider-compatible ARN generated
3. ❌ User is **NOT** created in AWS/GCP/Azure
4. ❌ No way to know if sync succeeded or failed
5. ❌ No way to retry failed syncs
6. ❌ No way to query sync status

This means WAMI is essentially a local-only IAM system without real cloud provider integration. Users need:
- Automatic creation in AWS IAM when creating a WAMI user with AWS provider enabled
- Sync status tracking to know if resources are synced
- Error handling when provider APIs fail
- Retry mechanisms for transient failures
- Optional sync (some resources may be WAMI-only)

## Proposed Solution

Implement a comprehensive provider synchronization system with:

1. **Sync State Enum** - Track resource state per provider:
   - `Pending` - Resource exists in WAMI but not synced yet
   - `Syncing` - Sync in progress
   - `Synced` - Successfully synced to provider
   - `Failed` - Sync failed (with error details)
   - `NotConfigured` - Provider not configured for this resource

2. **Provider Sync Service** - Orchestrates synchronization:
   - Async background sync (fire-and-forget)
   - Optional blocking sync (wait for completion)
   - Retry with exponential backoff
   - Batch sync operations
   - Conflict resolution (resource exists in provider)

3. **Provider Adapters** - Cloud provider-specific implementations:
   - AWS IAM adapter (using AWS SDK)
   - GCP IAM adapter (using GCP SDK)
   - Azure RBAC adapter (using Azure SDK)
   - Custom provider adapter (extension point)

4. **Sync Configuration** - Per-resource or per-tenant:
   - Which providers to sync to
   - Sync direction (WAMI → Provider, Provider → WAMI, bidirectional)
   - Sync triggers (on create, on update, on delete, scheduled)
   - Conflict resolution strategy

5. **Store Integration** - Sync state in resource model:
   - Update `ProviderConfig` with sync state
   - Add sync metadata (last error, retry count, next retry time)
   - Store-level sync state queries

## Sync State Model

### Enhanced ProviderConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncState {
    /// Resource not yet synced to this provider
    Pending,
    /// Sync operation in progress
    Syncing,
    /// Successfully synced to provider
    Synced {
        /// When sync completed successfully
        synced_at: DateTime<Utc>,
        /// Provider-specific resource ID (if available)
        provider_resource_id: Option<String>,
    },
    /// Sync failed (transient or permanent)
    Failed {
        /// Error that caused the failure
        error: SyncError,
        /// When the failure occurred
        failed_at: DateTime<Utc>,
        /// Number of retry attempts
        retry_count: u32,
        /// When to retry next (exponential backoff)
        next_retry_at: Option<DateTime<Utc>>,
    },
    /// Provider not configured for this resource/tenant
    NotConfigured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    /// Error code (e.g., "AlreadyExists", "AccessDenied", "NetworkError")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Whether this error is retryable
    pub retryable: bool,
    /// Original error details (provider-specific)
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    /// The provider name (e.g., "aws", "gcp", "azure", "custom")
    pub provider_name: String,
    /// The account/project/subscription identifier
    pub account_id: String,
    /// The provider-specific ARN/identifier (populated after sync)
    pub native_arn: Option<String>,
    /// Current sync state
    pub sync_state: SyncState,
    /// Optional tenant ID for multi-tenant isolation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}
```

## Implementation Plan

### Phase 1: Sync State Model (Week 1)

**Tasks**:
- [ ] Add `SyncState` enum to `src/provider/mod.rs`
- [ ] Add `SyncError` struct to `src/provider/mod.rs`
- [ ] Update `ProviderConfig` to include `sync_state` field
- [ ] Update all resource models (User, Role, Group, Policy) to use new `ProviderConfig`
- [ ] Migration utilities for existing resources (default to `SyncState::Pending`)
- [ ] Unit tests for sync state transitions

**Files to Modify**:
- `src/provider/mod.rs` - Add sync state types
- `src/wami/identity/user/model.rs` - Update to use new ProviderConfig
- `src/wami/identity/role/model.rs` - Update to use new ProviderConfig
- `src/wami/identity/group/model.rs` - Update to use new ProviderConfig
- `src/wami/policies/policy/model.rs` - Update to use new ProviderConfig

### Phase 2: Provider Adapter Trait (Week 2)

**Tasks**:
- [ ] Create `ProviderAdapter` trait in `src/provider/adapter.rs`
- [ ] Define methods: `create_user()`, `update_user()`, `delete_user()`, etc.
- [ ] Create `ProviderAdapterError` enum for provider-specific errors
- [ ] Define sync configuration (`SyncConfig`, `SyncDirection`, etc.)
- [ ] Unit tests for adapter trait

**Files to Create**:
- `src/provider/adapter.rs` - Trait definition

**Example Trait**:
```rust
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    /// Create a user in the cloud provider
    async fn create_user(
        &self,
        user: &User,
        config: &SyncConfig,
    ) -> Result<ProviderResource, ProviderAdapterError>;
    
    /// Update a user in the cloud provider
    async fn update_user(
        &self,
        user: &User,
        provider_resource_id: &str,
        config: &SyncConfig,
    ) -> Result<ProviderResource, ProviderAdapterError>;
    
    /// Delete a user from the cloud provider
    async fn delete_user(
        &self,
        provider_resource_id: &str,
        config: &SyncConfig,
    ) -> Result<(), ProviderAdapterError>;
    
    /// Similar methods for roles, groups, policies, etc.
}

pub struct ProviderResource {
    pub native_arn: String,
    pub provider_resource_id: Option<String>,
    pub synced_at: DateTime<Utc>,
}
```

### Phase 3: AWS Provider Adapter (Weeks 3-4)

**Tasks**:
- [ ] Add `aws-sdk-iam` dependency to `Cargo.toml`
- [ ] Implement `ProviderAdapter` for AWS in `src/provider/adapters/aws.rs`
- [ ] Handle AWS-specific errors (AlreadyExists, AccessDenied, etc.)
- [ ] Map WAMI resources to AWS IAM resources
- [ ] Handle AWS resource limits and validations
- [ ] Integration tests with AWS SDK (mocked or real)
- [ ] Error mapping (AWS errors → `ProviderAdapterError`)

**Files to Create**:
- `src/provider/adapters/mod.rs`
- `src/provider/adapters/aws.rs`

**Dependencies**:
```toml
[dependencies]
aws-sdk-iam = "1.0"
aws-config = "1.0"
```

### Phase 4: Provider Sync Service (Week 5)

**Tasks**:
- [ ] Create `ProviderSyncService` in `src/service/provider/sync.rs`
- [ ] Implement sync orchestration (create, update, delete)
- [ ] Implement retry logic with exponential backoff
- [ ] Implement batch sync operations
- [ ] Handle concurrent syncs (same resource, multiple providers)
- [ ] Store sync state updates
- [ ] Unit tests for sync service

**Files to Create**:
- `src/service/provider/mod.rs`
- `src/service/provider/sync.rs`

**Core Methods**:
```rust
pub struct ProviderSyncService<S> {
    store: Arc<RwLock<S>>,
    adapters: HashMap<String, Arc<dyn ProviderAdapter>>,
}

impl<S: UserStore + Send + Sync> ProviderSyncService<S> {
    /// Sync user to specified providers (async, fire-and-forget)
    pub async fn sync_user_async(
        &self,
        user: &User,
        provider_names: &[&str],
    ) -> Result<Vec<SyncResult>, SyncError>;
    
    /// Sync user to specified providers (blocking, wait for completion)
    pub async fn sync_user_blocking(
        &self,
        user: &User,
        provider_names: &[&str],
    ) -> Result<Vec<SyncResult>, SyncError>;
    
    /// Retry failed syncs
    pub async fn retry_failed_syncs(
        &self,
        resource_type: ResourceType,
        max_retries: u32,
    ) -> Result<RetrySummary>;
    
    /// Get sync status for a resource
    pub async fn get_sync_status(
        &self,
        resource_arn: &str,
    ) -> Result<HashMap<String, SyncState>>;
}
```

### Phase 5: Integration with User Service (Week 6)

**Tasks**:
- [ ] Update `UserService::create_user()` to trigger sync (optional)
- [ ] Update `UserService::update_user()` to trigger sync (optional)
- [ ] Update `UserService::delete_user()` to trigger sync (optional)
- [ ] Add sync configuration to `CreateUserRequest`
- [ ] Handle sync errors gracefully (don't fail user creation if sync fails)
- [ ] Update existing tests to work with sync (mock adapters)
- [ ] Integration tests

**Files to Modify**:
- `src/service/identity/user.rs` - Add sync calls
- `src/wami/identity/user/mod.rs` - Update CreateUserRequest

**Example Integration**:
```rust
impl<S: UserStore> UserService<S> {
    pub async fn create_user(
        &self,
        context: &WamiContext,
        request: CreateUserRequest,
        sync_service: Option<&ProviderSyncService<S>>, // Optional sync
    ) -> Result<User> {
        // ... existing user creation ...
        let user = self.store.write().unwrap().create_user(user).await?;
        
        // Trigger sync if configured
        if let Some(sync) = sync_service {
            if let Some(providers) = &request.sync_to_providers {
                let _ = sync.sync_user_async(&user, providers).await;
                // Don't fail user creation if sync fails
            }
        }
        
        Ok(user)
    }
}
```

### Phase 6: GCP and Azure Adapters (Weeks 7-8)

**Tasks**:
- [ ] Implement GCP adapter using Google Cloud IAM SDK
- [ ] Implement Azure adapter using Azure RBAC SDK
- [ ] Add dependencies: `google-cloud-iam`, `azure-identity`, `azure-rbac`
- [ ] Map WAMI resources to GCP service accounts
- [ ] Map WAMI resources to Azure AD users/service principals
- [ ] Handle provider-specific limitations
- [ ] Integration tests

**Files to Create**:
- `src/provider/adapters/gcp.rs`
- `src/provider/adapters/azure.rs`

**Dependencies**:
```toml
[dependencies]
google-cloud-iam = "1.0"
azure-identity = "1.0"
azure-rbac = "1.0"
```

### Phase 7: Sync Configuration and Management (Week 9)

**Tasks**:
- [ ] Create sync configuration store/manager
- [ ] Per-tenant sync configuration
- [ ] Per-resource sync overrides
- [ ] Sync schedule configuration (sync on create, on update, scheduled)
- [ ] Admin API for managing sync configs
- [ ] CLI commands for sync management

**Files to Create**:
- `src/service/provider/config.rs`
- `src/store/traits/provider/sync_config.rs`

### Phase 8: Monitoring and Observability (Week 10)

**Tasks**:
- [ ] Metrics for sync operations (success rate, latency, errors)
- [ ] Logging for sync operations
- [ ] Admin API for querying sync status
- [ ] Dashboard data structures for sync health
- [ ] Alerts for sync failures

**Files to Create**:
- `src/service/provider/metrics.rs`
- `src/service/provider/reports.rs`

### Phase 9: Documentation and Examples (Week 10)

**Tasks**:
- [ ] User guide for provider synchronization
- [ ] API reference for sync service
- [ ] Examples showing AWS/GCP/Azure sync
- [ ] Troubleshooting guide
- [ ] Migration guide (enabling sync for existing resources)
- [ ] Update README.md

**Files to Create**:
- `docs/PROVIDER_SYNC_GUIDE.md`
- `examples/27_provider_sync_aws.rs`
- `examples/28_provider_sync_multi_cloud.rs`

## Examples

### Example 1: Create User with AWS Sync

```rust
use wami::service::identity::UserService;
use wami::service::provider::ProviderSyncService;
use wami::wami::identity::user::CreateUserRequest;

// Setup AWS adapter
let aws_adapter = Arc::new(AwsAdapter::new(
    aws_config::load_from_env().await
));

// Create sync service
let mut sync_service = ProviderSyncService::new(store.clone());
sync_service.register_adapter("aws", aws_adapter);

// Create user service
let user_service = UserService::new(store.clone());

// Create user with sync
let request = CreateUserRequest {
    user_name: "alice".to_string(),
    path: Some("/engineering/".to_string()),
    permissions_boundary: None,
    tags: None,
    sync_to_providers: Some(vec!["aws".to_string()]), // Enable sync
};

let user = user_service
    .create_user(&context, request, Some(&sync_service))
    .await?;

// Check sync status
let sync_status = sync_service.get_sync_status(&user.wami_arn.to_string()).await?;
println!("AWS Sync Status: {:?}", sync_status.get("aws"));
// Output: SyncState::Synced { synced_at: ..., provider_resource_id: Some("AIDACKCEVSQ6C2EXAMPLE") }
```

### Example 2: Retry Failed Syncs

```rust
// User creation succeeded, but AWS sync failed (network issue)
// Later, retry failed syncs

let retry_summary = sync_service
    .retry_failed_syncs(ResourceType::User, max_retries: 3)
    .await?;

println!("Retried: {} successes, {} failures", 
    retry_summary.success_count, 
    retry_summary.failure_count);
```

### Example 3: Multi-Cloud Sync

```rust
// Register multiple providers
sync_service.register_adapter("aws", aws_adapter);
sync_service.register_adapter("gcp", gcp_adapter);
sync_service.register_adapter("azure", azure_adapter);

// Create user that syncs to all providers
let request = CreateUserRequest {
    user_name: "multi-cloud-user".to_string(),
    path: Some("/".to_string()),
    permissions_boundary: None,
    tags: None,
    sync_to_providers: Some(vec![
        "aws".to_string(),
        "gcp".to_string(),
        "azure".to_string(),
    ]),
};

let user = user_service
    .create_user(&context, request, Some(&sync_service))
    .await?;

// Check status for all providers
let sync_status = sync_service.get_sync_status(&user.wami_arn.to_string()).await?;
for (provider, state) in sync_status {
    println!("{}: {:?}", provider, state);
}
```

### Example 4: Sync Status Query

```rust
// Query sync status for a user
let sync_status = sync_service.get_sync_status(
    "arn:wami:iam:12345678:wami:123456789012:user/alice"
).await?;

match sync_status.get("aws") {
    Some(SyncState::Synced { synced_at, provider_resource_id }) => {
        println!("✅ Synced to AWS at {} with ID {:?}", 
            synced_at, provider_resource_id);
    }
    Some(SyncState::Failed { error, retry_count, next_retry_at, .. }) => {
        println!("❌ Sync failed: {} (retries: {}, next retry: {:?})",
            error.message, retry_count, next_retry_at);
    }
    Some(SyncState::Pending) => {
        println!("⏳ Sync pending...");
    }
    Some(SyncState::Syncing) => {
        println!("🔄 Sync in progress...");
    }
    _ => {}
}
```

### Example 5: Handle Sync Conflicts

```rust
// User already exists in AWS (created outside WAMI)
// WAMI tries to sync -> conflict

match sync_service.sync_user_blocking(&user, &["aws"]).await {
    Ok(results) => {
        // Success
    }
    Err(SyncError::Conflict { existing_resource }) => {
        // Handle conflict: merge, skip, or update?
        sync_service.resolve_conflict(
            &user.wami_arn.to_string(),
            "aws",
            ConflictResolution::MergeWithExisting(existing_resource),
        ).await?;
    }
    Err(e) => {
        // Other error
        eprintln!("Sync failed: {}", e);
    }
}
```

## Testing Strategy

1. **Unit Tests**:
   - Sync state transitions
   - Retry logic with exponential backoff
   - Error mapping (AWS → ProviderAdapterError)
   - Conflict resolution

2. **Integration Tests**:
   - Mock provider adapters (no real API calls)
   - Test sync service with mocked adapters
   - Test sync state persistence in store

3. **Provider Adapter Tests**:
   - AWS adapter with AWS SDK (use LocalStack or mocks)
   - GCP adapter with GCP SDK (use emulator or mocks)
   - Azure adapter with Azure SDK (use mocks)

4. **End-to-End Tests**:
   - Create user → sync to AWS → verify in AWS
   - Update user → sync update → verify in AWS
   - Delete user → sync deletion → verify removed from AWS
   - Handle sync failures gracefully
   - Retry failed syncs

5. **Performance Tests**:
   - Batch sync performance
   - Concurrent syncs
   - Sync latency benchmarks

## Success Criteria

- [ ] Sync state enum implemented and integrated into resource models
- [ ] AWS provider adapter creates/updates/deletes users in AWS IAM
- [ ] Sync service orchestrates sync operations with retry logic
- [ ] User creation optionally triggers sync to configured providers
- [ ] Sync status queryable per resource per provider
- [ ] Failed syncs retryable with exponential backoff
- [ ] GCP and Azure adapters implemented (optional but recommended)
- [ ] Comprehensive error handling (network errors, permission errors, etc.)
- [ ] Documentation and examples complete
- [ ] Test coverage > 80%

## Dependencies

### External Dependencies
- `aws-sdk-iam` - AWS IAM SDK for Rust
- `aws-config` - AWS configuration
- `google-cloud-iam` - GCP IAM SDK (optional)
- `azure-identity` + `azure-rbac` - Azure SDK (optional)

### Internal Dependencies
- Existing provider system (`CloudProvider` trait)
- Store traits (`UserStore`, `RoleStore`, etc.)
- Resource models (`User`, `Role`, `Group`, `Policy`)
- Error handling system

### AWS Credentials Configuration

Users will need to configure AWS credentials:
- Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
- AWS credentials file (`~/.aws/credentials`)
- IAM roles (for EC2/ECS/Lambda)
- AWS SSO

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Provider API rate limits | High | Implement rate limiting, exponential backoff, batch operations |
| Provider API failures (network, auth) | High | Retry logic, graceful degradation (don't fail resource creation if sync fails) |
| Sync state inconsistency | Medium | Transaction-like updates, sync state validation, repair utilities |
| Cost of provider API calls | Medium | Configurable sync (opt-in), batch operations, caching |
| Provider credential management | High | Secure credential storage, credential rotation support, per-tenant credentials |
| Conflict resolution complexity | Medium | Clear conflict resolution strategies (skip, merge, overwrite), admin tools |
| Performance impact | Medium | Async sync by default, optional blocking sync, background sync workers |
| Multi-cloud complexity | High | Phased approach (AWS first, then GCP/Azure), comprehensive testing |

## Configuration

### Sync Configuration Options

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Which providers to sync to (empty = sync to none)
    pub providers: Vec<String>,
    
    /// Sync direction
    pub direction: SyncDirection,
    
    /// Sync triggers
    pub triggers: SyncTriggers,
    
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
    
    /// Retry configuration
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDirection {
    /// WAMI → Provider (push)
    Push,
    /// Provider → WAMI (pull)
    Pull,
    /// Bidirectional sync
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTriggers {
    /// Sync on resource creation
    pub on_create: bool,
    /// Sync on resource update
    pub on_update: bool,
    /// Sync on resource deletion
    pub on_delete: bool,
    /// Scheduled sync interval (optional)
    pub scheduled: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Skip sync if resource exists in provider
    Skip,
    /// Overwrite provider resource with WAMI resource
    Overwrite,
    /// Merge provider resource with WAMI resource
    Merge,
    /// Fail sync with error
    Fail,
}
```

## Related Issues

- N/A (First provider sync issue)

## Related Documentation

- `docs/MULTICLOUD_PROVIDERS.md` - Provider abstraction overview
- `docs/MULTICLOUD_STATUS.md` - Multi-cloud implementation status
- `src/provider/mod.rs` - Provider system architecture
- `src/service/identity/user.rs` - User service implementation

## References

- [AWS IAM API Reference](https://docs.aws.amazon.com/IAM/latest/APIReference/)
- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
- [GCP IAM API Reference](https://cloud.google.com/iam/docs/reference/rest)
- [Azure RBAC API Reference](https://learn.microsoft.com/en-us/rest/api/authorization/)

---

**Estimated Effort**: 10 weeks (1 developer)  
**Estimated LOC**: ~5,000-7,000 lines of new code  
**Dependencies**: AWS SDK (required), GCP/Azure SDKs (optional)

