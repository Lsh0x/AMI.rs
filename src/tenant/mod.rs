//! Multi-Tenant Architecture Module
//!
//! This module provides hierarchical multi-tenancy support for WAMI,
//! allowing tenants to create sub-tenants with inherited quotas and permissions.
//!
//! # Features
//!
//! - Hierarchical tenant structure (unlimited depth with constraints)
//! - Per-tenant resource isolation
//! - Quota management with inheritance
//! - Permission-based access control
//! - Backward compatible (single-tenant mode when tenant_id is None)
//!
//! # Example
//!
//! ```rust,ignore
//! use wami::tenant::{TenantId, Tenant, TenantClient};
//!
//! // Create root tenant
//! let root_id = TenantId::root("acme");
//!
//! // Create child tenant
//! let child_id = root_id.child("engineering");
//!
//! // Check hierarchy
//! assert!(child_id.is_descendant_of(&root_id));
//! assert_eq!(child_id.depth(), 1);
//! ```

pub mod client;
pub mod model;
pub mod store;

#[cfg(test)]
mod tests;

pub use client::TenantClient;
pub use model::*;
pub use store::{InMemoryTenantStore, TenantStore};
