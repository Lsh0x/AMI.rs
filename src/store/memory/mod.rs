//! In-Memory Store Implementations
//!
//! This module contains in-memory implementations of all store traits.
//! These implementations are primarily used for testing and development.
//!
//! # Architecture Evolution
//!
//! ## Legacy Stores (Backward Compatibility)
//!
//! The original store implementations were service-specific:
//! - `InMemoryIamStore` - Separate HashMaps for users, roles, policies, groups, etc.
//! - `InMemoryStsStore` - Separate HashMaps for sessions and credentials
//! - `InMemoryTenantStore` - HashMap for tenants
//! - `InMemorySsoAdminStore` - SSO Admin data
//!
//! These are still available for backward compatibility.
//!
//! ## New Unified Store (Recommended)
//!
//! The new `UnifiedInMemoryStore` uses a single HashMap for all resources:
//! - Single data structure indexed by WAMI ARN
//! - Simpler implementation
//! - Better performance for cross-resource queries
//! - Easier to maintain and extend
//!
//! See the `unified_store` module for detailed documentation.

mod iam;
mod sso_admin;
mod sts;
mod tenant;
mod unified;
mod unified_store;

// Legacy stores (backward compatibility)
pub use iam::InMemoryIamStore;
pub use sso_admin::InMemorySsoAdminStore;
pub use sts::InMemoryStsStore;
pub use tenant::InMemoryTenantStore;
pub use unified::InMemoryStore;

// New unified store (recommended)
pub use unified_store::UnifiedInMemoryStore;
