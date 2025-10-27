//! Unified In-Memory Store Implementation
//!
//! # Overview
//!
//! This module provides a complete implementation of the unified `Store` trait
//! using a single in-memory `HashMap`. All resources (Users, Roles, Policies, etc.)
//! are stored together, indexed by their WAMI ARN.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    UnifiedInMemoryStore                     │
//! │                                                             │
//! │  RwLock<HashMap<String, Resource>>                          │
//! │                                                             │
//! │  Key Examples:                                              │
//! │  - "arn:wami:iam:a1b2c3:user/alice"       -> Resource::User │
//! │  - "arn:wami:iam:a1b2c3:role/admin"       -> Resource::Role │
//! │  - "arn:wami:sts:a1b2c3:session/sess-123" -> Resource::Sts  │
//! │  - "arn:wami:tenant:a1b2c3:tenant/main"   -> Resource::Ten  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Benefits
//!
//! 1. **Simplicity**: Single data structure for all resources
//! 2. **Performance**: O(1) lookups by ARN, no joins needed
//! 3. **Consistency**: All resources follow the same storage pattern
//! 4. **Flexibility**: Easy to add new resource types
//! 5. **Thread-Safe**: RwLock allows concurrent reads
//!
//! # Example Usage
//!
//! ```rust
//! use wami::store::memory::UnifiedInMemoryStore;
//! use wami::store::{Store, Resource};
//! use wami::iam::user::User;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = UnifiedInMemoryStore::new();
//!
//! // Store a user
//! // let user = User { arn: "arn:wami:iam:a1b2c3:user/alice".to_string(), /* ... */ };
//! // store.put(Resource::User(user)).await?;
//!
//! // Retrieve by ARN
//! if let Some(resource) = store.get("arn:wami:iam:a1b2c3:user/alice").await? {
//!     if let Some(user) = resource.as_user() {
//!         println!("Found user: {}", user.user_name);
//!     }
//! }
//!
//! // Query all users in tenant
//! let users = store.query("arn:wami:iam:a1b2c3:user/*").await?;
//! println!("Found {} users", users.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Thread Safety
//!
//! This implementation uses `RwLock` for interior mutability:
//! - Multiple concurrent reads are allowed
//! - Writes require exclusive access
//! - No blocking on read-heavy workloads
//!
//! # Migration from Legacy Stores
//!
//! If you're migrating from the old `IamStore`/`StsStore`/etc. traits:
//!
//! ```rust,ignore
//! // Old way
//! let user = iam_store.get_user("alice").await?;
//!
//! // New way
//! let arn = "arn:wami:iam:tenant-hash:user/alice";
//! let resource = store.get(arn).await?;
//! let user = resource.and_then(|r| r.as_user());
//! ```

use crate::error::{AmiError, Result};
use crate::store::resource::Resource;
use crate::store::traits::Store;
use async_trait::async_trait;
use regex;
use std::collections::HashMap;
use std::sync::RwLock;

/// Unified in-memory store implementation
///
/// All resources are stored in a single HashMap indexed by WAMI ARN.
///
/// # Thread Safety
///
/// This struct is thread-safe and can be shared across threads using `Arc`:
///
/// ```rust
/// use wami::store::memory::UnifiedInMemoryStore;
/// use std::sync::Arc;
///
/// let store = Arc::new(UnifiedInMemoryStore::new());
/// // Clone and share across threads
/// let store_clone = Arc::clone(&store);
/// ```
///
/// # Performance Characteristics
///
/// - `get()`: O(1) - Direct HashMap lookup
/// - `put()`: O(1) - Direct HashMap insert
/// - `delete()`: O(1) - Direct HashMap remove
/// - `query()`: O(n) - Must scan all keys and match pattern
///   - With tenant-specific queries: O(m) where m = resources in tenant
///   - Optimization: Index by tenant hash for faster queries
///
/// # Memory Usage
///
/// Each resource is stored once in the HashMap. Memory usage is proportional
/// to the number of resources stored.
///
/// Estimated memory per resource:
/// - User: ~500 bytes (without policies)
/// - Role: ~400 bytes (without policies)
/// - Policy: ~1-5 KB (depending on document size)
/// - Access Key: ~200 bytes
/// - ARN string: ~80 bytes
///
/// Example: 10,000 users ≈ 5 MB
#[derive(Debug, Default)]
pub struct UnifiedInMemoryStore {
    /// The main storage: ARN -> Resource
    ///
    /// Using RwLock for interior mutability:
    /// - Allows `&self` methods (no `&mut self` needed)
    /// - Supports concurrent reads
    /// - Writes are serialized
    resources: RwLock<HashMap<String, Resource>>,
}

impl UnifiedInMemoryStore {
    /// Creates a new empty store
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::store::memory::UnifiedInMemoryStore;
    ///
    /// let store = UnifiedInMemoryStore::new();
    /// ```
    pub fn new() -> Self {
        Self {
            resources: RwLock::new(HashMap::new()),
        }
    }

    /// Creates a store with pre-allocated capacity
    ///
    /// Use this when you know approximately how many resources you'll store
    /// to avoid repeated HashMap reallocations.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Initial capacity (number of resources)
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::store::memory::UnifiedInMemoryStore;
    ///
    /// // Pre-allocate for 10,000 resources
    /// let store = UnifiedInMemoryStore::with_capacity(10_000);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            resources: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    /// Gets the current number of resources in the store
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::store::memory::UnifiedInMemoryStore;
    ///
    /// let store = UnifiedInMemoryStore::new();
    /// assert_eq!(store.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.resources.read().unwrap().len()
    }

    /// Checks if the store is empty
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::store::memory::UnifiedInMemoryStore;
    ///
    /// let store = UnifiedInMemoryStore::new();
    /// assert!(store.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.resources.read().unwrap().is_empty()
    }

    /// Clears all resources from the store
    ///
    /// # Warning
    ///
    /// This is a destructive operation! All data will be lost.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::store::memory::UnifiedInMemoryStore;
    ///
    /// let store = UnifiedInMemoryStore::new();
    /// // ... add resources ...
    /// store.clear();
    /// assert!(store.is_empty());
    /// ```
    pub fn clear(&self) {
        self.resources.write().unwrap().clear();
    }

    /// Helper method to match ARN patterns
    ///
    /// Converts wildcard patterns to regex and matches against ARNs.
    ///
    /// # Pattern Syntax
    ///
    /// - `*` matches any sequence of characters (including `/` and `:`)
    /// - `?` matches any single character
    ///
    /// # Examples
    ///
    /// ```text
    /// "arn:wami:iam:a1b2c3:user/*"      matches "arn:wami:iam:a1b2c3:user/alice"
    /// "arn:wami:iam:*:user/admin*"     matches "arn:wami:iam:x:user/admin-alice"
    /// "arn:wami:iam:a1b2c3:role/?"     matches "arn:wami:iam:a1b2c3:role/x"
    /// ```
    fn matches_pattern(arn: &str, pattern: &str) -> bool {
        // Use direct regex matching instead of ParsedArn
        // Convert IAM-style wildcards to regex
        let escaped = regex::escape(pattern);
        let with_wildcards = escaped.replace(r"\*", ".*").replace(r"\?", ".");
        let regex_pattern = format!("^{}$", with_wildcards);

        if let Ok(re) = regex::Regex::new(&regex_pattern) {
            re.is_match(arn)
        } else {
            false
        }
    }
}

#[async_trait]
impl Store for UnifiedInMemoryStore {
    /// Gets a resource by exact ARN
    ///
    /// # Performance
    ///
    /// O(1) - Direct HashMap lookup
    ///
    /// # Errors
    ///
    /// - Returns error if RwLock is poisoned (should never happen in normal use)
    async fn get(&self, arn: &str) -> Result<Option<Resource>> {
        let resources = self
            .resources
            .read()
            .map_err(|e| AmiError::StoreError(format!("Lock poisoned: {}", e)))?;

        Ok(resources.get(arn).cloned())
    }

    /// Queries resources matching an ARN pattern
    ///
    /// # Performance
    ///
    /// O(n) where n = total number of resources in store
    ///
    /// For better performance with large stores:
    /// - Use specific tenant hash instead of wildcard
    /// - Use specific resource type instead of wildcard
    /// - Consider adding secondary indexes
    ///
    /// # Implementation Notes
    ///
    /// This implementation scans all ARNs and matches each against the pattern.
    /// For production use with large datasets, consider:
    /// - Adding a secondary index by tenant hash
    /// - Adding a secondary index by resource type
    /// - Using a database with proper indexing
    ///
    /// # Errors
    ///
    /// - Returns error if RwLock is poisoned
    async fn query(&self, pattern: &str) -> Result<Vec<Resource>> {
        let resources = self
            .resources
            .read()
            .map_err(|e| AmiError::StoreError(format!("Lock poisoned: {}", e)))?;

        let mut results = Vec::new();

        // Iterate through all resources and match pattern
        for (arn, resource) in resources.iter() {
            if Self::matches_pattern(arn, pattern) {
                results.push(resource.clone());
            }
        }

        Ok(results)
    }

    /// Stores a resource (create or update)
    ///
    /// # Performance
    ///
    /// O(1) - Direct HashMap insert
    ///
    /// # Behavior
    ///
    /// - Extracts ARN from the resource
    /// - Overwrites existing resource with same ARN
    /// - Creates new entry if ARN doesn't exist
    ///
    /// # Errors
    ///
    /// - Returns error if RwLock is poisoned
    async fn put(&self, resource: Resource) -> Result<()> {
        let arn = resource.arn().to_string();

        let mut resources = self
            .resources
            .write()
            .map_err(|e| AmiError::StoreError(format!("Lock poisoned: {}", e)))?;

        resources.insert(arn, resource);

        Ok(())
    }

    /// Deletes a resource by ARN
    ///
    /// # Performance
    ///
    /// O(1) - Direct HashMap remove
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if resource existed and was deleted
    /// - `Ok(false)` if resource did not exist
    ///
    /// # Errors
    ///
    /// - Returns error if RwLock is poisoned
    async fn delete(&self, arn: &str) -> Result<bool> {
        let mut resources = self
            .resources
            .write()
            .map_err(|e| AmiError::StoreError(format!("Lock poisoned: {}", e)))?;

        Ok(resources.remove(arn).is_some())
    }

    /// Checks if a resource exists (optimized implementation)
    ///
    /// More efficient than `get()` because it doesn't clone the resource.
    ///
    /// # Performance
    ///
    /// O(1) - Direct HashMap contains_key
    async fn exists(&self, arn: &str) -> Result<bool> {
        let resources = self
            .resources
            .read()
            .map_err(|e| AmiError::StoreError(format!("Lock poisoned: {}", e)))?;

        Ok(resources.contains_key(arn))
    }

    /// Counts total resources (optimized implementation)
    ///
    /// More efficient than querying and counting results.
    ///
    /// # Performance
    ///
    /// O(1) - Direct HashMap len
    async fn count_all(&self) -> Result<usize> {
        Ok(self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::user::User;

    /// Helper to create a test user
    fn create_test_user(arn: &str, name: &str) -> Resource {
        Resource::User(User {
            arn: arn.to_string(),
            user_name: name.to_string(),
            user_id: format!("AIDA{}", name.to_uppercase()),
            path: "/".to_string(),
            create_date: chrono::Utc::now(),
            password_last_used: None,
            permissions_boundary: None,
            tags: Vec::new(),
            wami_arn: arn.to_string(),
            providers: Vec::new(),
            tenant_id: None,
        })
    }

    #[tokio::test]
    async fn test_new_store_is_empty() {
        let store = UnifiedInMemoryStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[tokio::test]
    async fn test_put_and_get() {
        let store = UnifiedInMemoryStore::new();
        let arn = "arn:wami:iam:a1b2c3:user/alice";
        let user = create_test_user(arn, "alice");

        // Put resource
        store.put(user.clone()).await.unwrap();

        // Get resource
        let retrieved = store.get(arn).await.unwrap();
        assert!(retrieved.is_some());

        let resource = retrieved.unwrap();
        let retrieved_user = resource.as_user().unwrap();
        assert_eq!(retrieved_user.user_name, "alice");
        assert_eq!(retrieved_user.arn, arn);
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let store = UnifiedInMemoryStore::new();
        let result = store
            .get("arn:wami:iam:a1b2c3:user/nonexistent")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete() {
        let store = UnifiedInMemoryStore::new();
        let arn = "arn:wami:iam:a1b2c3:user/alice";
        let user = create_test_user(arn, "alice");

        // Put and verify
        store.put(user).await.unwrap();
        assert!(store.exists(arn).await.unwrap());

        // Delete
        let deleted = store.delete(arn).await.unwrap();
        assert!(deleted); // Should return true (resource existed)
        assert!(!store.exists(arn).await.unwrap());

        // Delete again
        let deleted_again = store.delete(arn).await.unwrap();
        assert!(!deleted_again); // Should return false (resource didn't exist)
    }

    #[tokio::test]
    async fn test_query_wildcard() {
        let store = UnifiedInMemoryStore::new();

        // Add multiple users
        store
            .put(create_test_user("arn:wami:iam:a1b2c3:user/alice", "alice"))
            .await
            .unwrap();
        store
            .put(create_test_user("arn:wami:iam:a1b2c3:user/bob", "bob"))
            .await
            .unwrap();
        store
            .put(create_test_user(
                "arn:wami:iam:xyz789:user/charlie",
                "charlie",
            ))
            .await
            .unwrap();

        // Query all users in tenant a1b2c3
        let results = store.query("arn:wami:iam:a1b2c3:user/*").await.unwrap();
        assert_eq!(results.len(), 2);

        // Query all users across tenants
        let all_users = store.query("arn:wami:iam:*:user/*").await.unwrap();
        assert_eq!(all_users.len(), 3);

        // Query specific tenant
        let tenant_users = store.query("arn:wami:iam:xyz789:*").await.unwrap();
        assert_eq!(tenant_users.len(), 1);
    }

    #[tokio::test]
    async fn test_list_tenant_resources() {
        let store = UnifiedInMemoryStore::new();

        // Add resources to different tenants
        store
            .put(create_test_user("arn:wami:iam:tenant1:user/alice", "alice"))
            .await
            .unwrap();
        store
            .put(create_test_user("arn:wami:iam:tenant1:user/bob", "bob"))
            .await
            .unwrap();
        store
            .put(create_test_user(
                "arn:wami:iam:tenant2:user/charlie",
                "charlie",
            ))
            .await
            .unwrap();

        // List tenant1 resources
        let tenant1_resources = store.list_tenant_resources("tenant1").await.unwrap();
        assert_eq!(tenant1_resources.len(), 2);

        // List tenant2 resources
        let tenant2_resources = store.list_tenant_resources("tenant2").await.unwrap();
        assert_eq!(tenant2_resources.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_tenant_resources() {
        let store = UnifiedInMemoryStore::new();

        // Add resources to different tenants
        store
            .put(create_test_user("arn:wami:iam:tenant1:user/alice", "alice"))
            .await
            .unwrap();
        store
            .put(create_test_user("arn:wami:iam:tenant1:user/bob", "bob"))
            .await
            .unwrap();
        store
            .put(create_test_user(
                "arn:wami:iam:tenant2:user/charlie",
                "charlie",
            ))
            .await
            .unwrap();

        // Delete all tenant1 resources
        let deleted = store.delete_tenant_resources("tenant1").await.unwrap();
        assert_eq!(deleted, 2);

        // Verify tenant1 is empty
        let tenant1_resources = store.list_tenant_resources("tenant1").await.unwrap();
        assert_eq!(tenant1_resources.len(), 0);

        // Verify tenant2 is unchanged
        let tenant2_resources = store.list_tenant_resources("tenant2").await.unwrap();
        assert_eq!(tenant2_resources.len(), 1);
    }

    #[tokio::test]
    async fn test_count() {
        let store = UnifiedInMemoryStore::new();

        assert_eq!(store.count_all().await.unwrap(), 0);

        store
            .put(create_test_user("arn:wami:iam:tenant1:user/alice", "alice"))
            .await
            .unwrap();
        store
            .put(create_test_user("arn:wami:iam:tenant1:user/bob", "bob"))
            .await
            .unwrap();

        assert_eq!(store.count_all().await.unwrap(), 2);
        assert_eq!(store.count_tenant("tenant1").await.unwrap(), 2);
        assert_eq!(store.count_tenant("tenant2").await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_clear() {
        let store = UnifiedInMemoryStore::new();

        store
            .put(create_test_user("arn:wami:iam:a1b2c3:user/alice", "alice"))
            .await
            .unwrap();
        assert_eq!(store.len(), 1);

        store.clear();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[tokio::test]
    async fn test_put_batch() {
        let store = UnifiedInMemoryStore::new();

        let resources = vec![
            create_test_user("arn:wami:iam:a1b2c3:user/alice", "alice"),
            create_test_user("arn:wami:iam:a1b2c3:user/bob", "bob"),
            create_test_user("arn:wami:iam:a1b2c3:user/charlie", "charlie"),
        ];

        let count = store.put_batch(resources).await.unwrap();
        assert_eq!(count, 3);
        assert_eq!(store.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_batch() {
        let store = UnifiedInMemoryStore::new();

        // Add users
        store
            .put(create_test_user("arn:wami:iam:a1b2c3:user/alice", "alice"))
            .await
            .unwrap();
        store
            .put(create_test_user("arn:wami:iam:a1b2c3:user/bob", "bob"))
            .await
            .unwrap();
        store
            .put(create_test_user(
                "arn:wami:iam:a1b2c3:user/charlie",
                "charlie",
            ))
            .await
            .unwrap();

        // Delete multiple
        let arns = vec![
            "arn:wami:iam:a1b2c3:user/alice".to_string(),
            "arn:wami:iam:a1b2c3:user/bob".to_string(),
            "arn:wami:iam:a1b2c3:user/nonexistent".to_string(), // Doesn't exist
        ];

        let deleted = store.delete_batch(arns).await.unwrap();
        assert_eq!(deleted, 2); // Only 2 existed

        assert_eq!(store.len(), 1); // Charlie remains
    }

    #[tokio::test]
    async fn test_list_by_type() {
        let store = UnifiedInMemoryStore::new();

        store
            .put(create_test_user("arn:wami:iam:a1b2c3:user/alice", "alice"))
            .await
            .unwrap();
        store
            .put(create_test_user("arn:wami:iam:a1b2c3:user/bob", "bob"))
            .await
            .unwrap();

        let users = store.list_by_type("a1b2c3", "user").await.unwrap();
        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    async fn test_overwrite() {
        let store = UnifiedInMemoryStore::new();
        let arn = "arn:wami:iam:a1b2c3:user/alice";

        // Create first version
        let user_v1 = create_test_user(arn, "alice_v1");
        store.put(user_v1).await.unwrap();

        let retrieved = store.get(arn).await.unwrap().unwrap();
        assert_eq!(retrieved.as_user().unwrap().user_name, "alice_v1");

        // Overwrite with second version
        let user_v2 = create_test_user(arn, "alice_v2");
        store.put(user_v2).await.unwrap();

        let retrieved = store.get(arn).await.unwrap().unwrap();
        assert_eq!(retrieved.as_user().unwrap().user_name, "alice_v2");

        // Should still have only one resource
        assert_eq!(store.len(), 1);
    }
}
