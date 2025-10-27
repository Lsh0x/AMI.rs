//! Unified Store Trait - ARN-Centric Architecture
//!
//! # Overview
//!
//! This trait provides a simplified, ARN-based interface for all store operations.
//! Instead of separate traits for IAM, STS, SSO Admin, and Tenant operations,
//! this unified trait handles all resource types through generic methods.
//!
//! # Why ARN-Centric?
//!
//! 1. **Security**: WAMI ARNs use hashed tenant IDs, preventing information leakage in logs
//! 2. **Multi-Cloud**: ARNs work consistently across AWS, GCP, Azure, and custom providers
//! 3. **Multi-Tenant**: Each resource is scoped to a tenant via the ARN's tenant hash
//! 4. **Simplicity**: One `get()` method replaces dozens of specific getters
//! 5. **Flexibility**: Query patterns enable powerful filtering (e.g., "arn:wami:iam:*:user/*")
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    WAMI ARN Format                          │
//! │  arn:wami:service:tenant-hash:resource-type/path/name       │
//! │                                                             │
//! │  - service: iam, sts, sso-admin, tenant                     │
//! │  - tenant-hash: SHA-256 hash of tenant ID (opaque)          │
//! │  - resource-type: user, role, policy, group, etc.           │
//! │  - path/name: hierarchical resource identifier              │
//! └─────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Unified Store Trait                       │
//! │                                                             │
//! │  get(arn) -> Option<Resource>                               │
//! │  query(pattern) -> Vec<Resource>                            │
//! │  put(resource) -> Result<()>                                │
//! │  delete(arn) -> Result<bool>                                │
//! │                                                             │
//! │  + tenant-aware methods                                     │
//! │  + bulk operations                                          │
//! └─────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────┐
//! │               Implementation (InMemoryStore)                │
//! │                                                             │
//! │  HashMap<String, Resource>                                  │
//! │    - Key: WAMI ARN (string)                                 │
//! │    - Value: Resource enum                                   │
//! │                                                             │
//! │  - Single data structure for all resources                  │
//! │  - O(1) lookups by ARN                                      │
//! │  - Pattern matching via regex on ARN keys                   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use wami::store::traits::Store;
//! use wami::store::resource::Resource;
//! use wami::store::memory::UnifiedInMemoryStore;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = UnifiedInMemoryStore::new();
//!
//! // 1. Get resource by exact ARN
//! if let Some(resource) = store.get("arn:wami:iam:a1b2c3:user/admin/alice").await? {
//!     match resource {
//!         Resource::User(user) => println!("User: {}", user.user_name),
//!         _ => println!("Other resource type"),
//!     }
//! }
//!
//! // 2. Query with wildcard patterns
//! // Find all users in tenant a1b2c3
//! let users = store.query("arn:wami:iam:a1b2c3:user/*").await?;
//!
//! // Find all admin users across all tenants
//! let admins = store.query("arn:wami:iam:*:user/admin/*").await?;
//!
//! // Find all roles in production tenant
//! let roles = store.query("arn:wami:iam:prod-hash:role/*").await?;
//!
//! // 3. Put (create or update) resource
//! // store.put(Resource::User(user)).await?;
//!
//! // 4. Delete resource
//! let deleted = store.delete("arn:wami:iam:a1b2c3:user/admin/alice").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security Considerations
//!
//! ## Opaque Tenant IDs
//!
//! WAMI ARNs use hashed tenant IDs to prevent information leakage:
//!
//! ```text
//! Real Tenant ID:  "acme-corp-production"
//! Hashed in ARN:   "a1b2c3d4e5f6"  (SHA-256 with salt)
//!
//! ✓ Safe to log: arn:wami:iam:a1b2c3d4e5f6:user/alice
//! ✗ Leaks info:  arn:wami:iam:acme-corp-production:user/alice
//! ```
//!
//! ## Database Indexing
//!
//! To efficiently query by tenant without leaking information:
//!
//! 1. **Primary Key**: WAMI ARN (opaque, includes tenant hash)
//! 2. **Secondary Index**: `tenant_hash` field extracted from ARN
//! 3. **Query**: `SELECT * WHERE tenant_hash = hash(tenant_id)`
//!
//! This allows fast tenant-scoped queries while maintaining opacity:
//!
//! ```text
//! Application has: tenant_id = "acme-corp"
//! Compute hash:    tenant_hash = hash("acme-corp") = "a1b2c3"
//! Query DB:        WHERE arn LIKE 'arn:wami:%:a1b2c3:%'
//! ```
//!
//! ## Multi-Provider Mapping
//!
//! Each WAMI resource can map to multiple cloud providers:
//!
//! ```text
//! WAMI ARN:     arn:wami:iam:a1b2c3:user/alice
//! AWS ARN:      arn:aws:iam::123456789012:user/alice
//! GCP Name:     projects/my-project/serviceAccounts/alice@project.iam
//! Azure URI:    /subscriptions/xxx/resourceGroups/rg/providers/...
//! ```
//!
//! The `ProviderInfo` field in resources stores these native identifiers.

use crate::error::Result;
use crate::store::resource::Resource;
use async_trait::async_trait;

/// Unified Store Trait - ARN-based operations for all resource types
///
/// # Design Philosophy
///
/// This trait represents a fundamental shift from type-specific operations
/// (e.g., `get_user`, `get_role`, `get_policy`) to generic ARN-based operations.
///
/// ## Before (Type-Specific)
///
/// ```text
/// trait IamStore {
///     fn get_user(user_name: &str) -> Option<User>;
///     fn list_users() -> Vec<User>;
///     fn create_user(user: User) -> Result<()>;
/// }
/// trait StsStore { /* similar */ }
/// trait TenantStore { /* similar */ }
/// ```
///
/// Problems:
/// - Each resource type needs dedicated methods
/// - No consistent way to handle ARNs
/// - Difficult to implement cross-cutting concerns (auditing, caching, etc.)
/// - Tenant isolation logic scattered across methods
///
/// ## After (ARN-Based)
///
/// ```text
/// trait Store {
///     fn get(arn: &str) -> Option<Resource>;
///     fn query(pattern: &str) -> Vec<Resource>;
///     fn put(resource: Resource) -> Result<()>;
///     fn delete(arn: &str) -> Result<bool>;
/// }
/// ```
///
/// Benefits:
/// - Single interface for all resource types
/// - ARN encodes tenant, service, type, and identity
/// - Easy to add new resource types
/// - Consistent behavior across all operations
///
/// # Thread Safety
///
/// All methods take `&self` (shared reference) because:
/// - Interior mutability is handled by the implementation (e.g., `RwLock`)
/// - Allows concurrent reads
/// - Implementations can choose their own synchronization strategy
///
#[async_trait]
pub trait Store: Send + Sync {
    // ==================== Core CRUD Operations ====================

    /// Gets a resource by its exact ARN
    ///
    /// # Arguments
    ///
    /// * `arn` - The complete WAMI ARN of the resource
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Resource))` - Resource found
    /// * `Ok(None)` - Resource not found
    /// * `Err(_)` - Storage error or invalid ARN format
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wami::store::traits::Store;
    /// # use wami::store::resource::Resource;
    /// # async fn example<S: Store>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get a specific user
    /// let user_arn = "arn:wami:iam:a1b2c3:user/admin/alice";
    /// if let Some(resource) = store.get(user_arn).await? {
    ///     if let Some(user) = resource.as_user() {
    ///         println!("Found user: {}", user.user_name);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Security Notes
    ///
    /// - ARN contains hashed tenant ID (opaque)
    /// - Safe to log ARN without leaking tenant name
    /// - Tenant isolation enforced at ARN level
    async fn get(&self, arn: &str) -> Result<Option<Resource>>;

    /// Queries resources matching an ARN pattern
    ///
    /// # Pattern Syntax
    ///
    /// - `*` - Matches any sequence of characters within a segment
    /// - `?` - Matches any single character
    /// - Patterns are matched against the full ARN string
    ///
    /// # Pattern Examples
    ///
    /// ```text
    /// "arn:wami:iam:a1b2c3:user/*"              // All users in tenant a1b2c3
    /// "arn:wami:iam:*:user/admin/*"            // All admin users (any tenant)
    /// "arn:wami:iam:a1b2c3:role/service-*"     // Service roles in tenant a1b2c3
    /// "arn:wami:iam:a1b2c3:*"                  // All IAM resources in tenant
    /// "arn:wami:*:a1b2c3:*"                    // All resources in tenant
    /// "arn:wami:sts:a1b2c3:session/*"          // All STS sessions
    /// ```
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Resource>)` - List of matching resources (empty if none match)
    /// * `Err(_)` - Storage error or invalid pattern
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wami::store::traits::Store;
    /// # use wami::store::resource::Resource;
    /// # async fn example<S: Store>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all users in a tenant
    /// let users = store.query("arn:wami:iam:a1b2c3:user/*").await?;
    /// for resource in users {
    ///     if let Some(user) = resource.as_user() {
    ///         println!("User: {}", user.user_name);
    ///     }
    /// }
    ///
    /// // Find all admin users across tenants (be careful with this!)
    /// let admins = store.query("arn:wami:iam:*:user/admin/*").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// - Query patterns with specific tenant hash are most efficient
    /// - Wildcard tenant (`arn:wami:iam:*:...`) scans all tenants
    /// - Implementations should optimize common patterns
    /// - Consider pagination for large result sets
    ///
    /// # Security Notes
    ///
    /// - Caller must have permission to query across tenants if using `*`
    /// - Results are pre-filtered by implementation based on access control
    /// - Always validate caller has permission before exposing results
    async fn query(&self, pattern: &str) -> Result<Vec<Resource>>;

    /// Stores a resource (create or update)
    ///
    /// # Behavior
    ///
    /// - If a resource with the same ARN exists, it is overwritten
    /// - If no resource exists, a new one is created
    /// - ARN is extracted from the resource itself
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to store (User, Role, Policy, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Resource stored successfully
    /// * `Err(_)` - Storage error or validation failure
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wami::store::traits::Store;
    /// # use wami::store::resource::Resource;
    /// # use wami::iam::user::User;
    /// # async fn example<S: Store>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    /// // Store a user
    /// // let user = User { /* ... */ };
    /// // store.put(Resource::User(user)).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Validation
    ///
    /// Implementations should validate:
    /// - ARN format is correct
    /// - Tenant hash in ARN matches the expected format
    /// - Resource-specific constraints (e.g., unique user names within tenant)
    ///
    /// # Atomicity
    ///
    /// Implementations should provide atomic put operations:
    /// - Either the entire resource is stored, or nothing changes
    /// - Concurrent puts to the same ARN should be serialized
    async fn put(&self, resource: Resource) -> Result<()>;

    /// Deletes a resource by ARN
    ///
    /// # Arguments
    ///
    /// * `arn` - The complete WAMI ARN of the resource to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Resource was deleted
    /// * `Ok(false)` - Resource did not exist
    /// * `Err(_)` - Storage error or invalid ARN
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wami::store::traits::Store;
    /// # async fn example<S: Store>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    /// let arn = "arn:wami:iam:a1b2c3:user/alice";
    /// if store.delete(arn).await? {
    ///     println!("User deleted");
    /// } else {
    ///     println!("User not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Cascading Deletes
    ///
    /// Implementations should handle related resources:
    /// - Deleting a user should delete their access keys, MFA devices, etc.
    /// - Deleting a role should delete its instance profiles
    /// - Consider soft deletes for audit trails
    ///
    /// # Atomicity
    ///
    /// Delete operations should be atomic and idempotent:
    /// - Multiple deletes of the same ARN should succeed (return false after first)
    /// - Cascading deletes should be all-or-nothing
    async fn delete(&self, arn: &str) -> Result<bool>;

    // ==================== Tenant-Scoped Operations ====================

    /// Lists all resources in a specific tenant
    ///
    /// This is a convenience method equivalent to:
    /// ```text
    /// query("arn:wami:*:tenant_hash:*")
    /// ```
    ///
    /// # Arguments
    ///
    /// * `tenant_hash` - The hashed tenant ID (from ARN)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wami::store::traits::Store;
    /// # async fn example<S: Store>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    /// let resources = store.list_tenant_resources("a1b2c3").await?;
    /// println!("Tenant has {} resources", resources.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn list_tenant_resources(&self, tenant_hash: &str) -> Result<Vec<Resource>> {
        let pattern = format!("arn:wami:*:{}:*", tenant_hash);
        self.query(&pattern).await
    }

    /// Deletes all resources in a specific tenant
    ///
    /// # Warning
    ///
    /// This is a destructive operation! Use with extreme caution.
    ///
    /// # Arguments
    ///
    /// * `tenant_hash` - The hashed tenant ID (from ARN)
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of resources deleted
    /// * `Err(_)` - Storage error (some resources may have been deleted)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wami::store::traits::Store;
    /// # async fn example<S: Store>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    /// // Delete all resources for tenant "a1b2c3"
    /// let deleted = store.delete_tenant_resources("a1b2c3").await?;
    /// println!("Deleted {} resources", deleted);
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_tenant_resources(&self, tenant_hash: &str) -> Result<usize> {
        let resources = self.list_tenant_resources(tenant_hash).await?;
        let mut count = 0;
        for resource in resources {
            if self.delete(resource.arn()).await? {
                count += 1;
            }
        }
        Ok(count)
    }

    // ==================== Bulk Operations ====================

    /// Stores multiple resources in a single operation
    ///
    /// # Arguments
    ///
    /// * `resources` - Vector of resources to store
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of resources successfully stored
    /// * `Err(_)` - Storage error (some resources may have been stored)
    ///
    /// # Atomicity
    ///
    /// Implementations should prefer atomic bulk operations, but may
    /// fall back to sequential puts if batch operations are not supported.
    async fn put_batch(&self, resources: Vec<Resource>) -> Result<usize> {
        let mut count = 0;
        for resource in resources {
            self.put(resource).await?;
            count += 1;
        }
        Ok(count)
    }

    /// Deletes multiple resources by ARN
    ///
    /// # Arguments
    ///
    /// * `arns` - Vector of ARNs to delete
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of resources actually deleted (existed)
    /// * `Err(_)` - Storage error (some resources may have been deleted)
    async fn delete_batch(&self, arns: Vec<String>) -> Result<usize> {
        let mut count = 0;
        for arn in arns {
            if self.delete(&arn).await? {
                count += 1;
            }
        }
        Ok(count)
    }

    // ==================== Resource Type Filters ====================

    /// Queries resources of a specific type in a tenant
    ///
    /// # Arguments
    ///
    /// * `tenant_hash` - The hashed tenant ID
    /// * `resource_type` - The resource type (e.g., "user", "role", "policy")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wami::store::traits::Store;
    /// # async fn example<S: Store>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get all users in tenant
    /// let users = store.list_by_type("a1b2c3", "user").await?;
    ///
    /// // Get all roles in tenant
    /// let roles = store.list_by_type("a1b2c3", "role").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn list_by_type(&self, tenant_hash: &str, resource_type: &str) -> Result<Vec<Resource>> {
        let pattern = format!("arn:wami:iam:{}:{}/*", tenant_hash, resource_type);
        self.query(&pattern).await
    }

    /// Queries resources of a specific type across all tenants
    ///
    /// # Warning
    ///
    /// This method queries across tenant boundaries. Ensure the caller
    /// has appropriate permissions before using this method.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - The resource type (e.g., "user", "role", "policy")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wami::store::traits::Store;
    /// # async fn example<S: Store>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get all users across all tenants (admin operation)
    /// let all_users = store.list_by_type_global("user").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn list_by_type_global(&self, resource_type: &str) -> Result<Vec<Resource>> {
        let pattern = format!("arn:wami:iam:*:{}/*", resource_type);
        self.query(&pattern).await
    }

    // ==================== Administrative Operations ====================

    /// Counts total resources in the store
    ///
    /// Useful for monitoring and capacity planning.
    async fn count_all(&self) -> Result<usize> {
        let all = self.query("arn:wami:*:*:*").await?;
        Ok(all.len())
    }

    /// Counts resources in a specific tenant
    ///
    /// # Arguments
    ///
    /// * `tenant_hash` - The hashed tenant ID
    async fn count_tenant(&self, tenant_hash: &str) -> Result<usize> {
        let resources = self.list_tenant_resources(tenant_hash).await?;
        Ok(resources.len())
    }

    /// Checks if a resource exists
    ///
    /// More efficient than `get()` if you only need to check existence.
    ///
    /// # Arguments
    ///
    /// * `arn` - The ARN to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Resource exists
    /// * `Ok(false)` - Resource does not exist
    /// * `Err(_)` - Storage error
    async fn exists(&self, arn: &str) -> Result<bool> {
        Ok(self.get(arn).await?.is_some())
    }
}
