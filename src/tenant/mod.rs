//! Tenant Module
//!
//! This module provides multi-tenant functionality for WAMI, including hierarchical
//! tenant management, resource isolation, and quota enforcement.
//!
//! # Overview
//!
//! Tenants can be organized in a hierarchy (e.g., "acme/engineering/frontend")
//! with cascading permissions and quota management.
//!
//! # Example
//!
//! ```rust,ignore
//! use wami::tenant::{TenantId, Tenant, TenantType};
//!
//! // Create root tenant
//! let root_id = TenantId::root("acme");
//!
//! // Create child tenant
//! let child_id = root_id.child("engineering");
//! assert_eq!(child_id.as_str(), "acme/engineering");
//! ```

pub mod authorization;
pub mod client;
pub mod hierarchy;
pub mod model;

#[cfg(test)]
mod tests;

pub use authorization::{
    build_tenant_admin_policy, build_tenant_readonly_policy, check_tenant_permission, TenantAction,
    TenantAuthorizer,
};
pub use client::TenantClient;
pub use model::*;
