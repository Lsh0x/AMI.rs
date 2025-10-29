//! In-Memory Store Implementations
//!
//! This module contains in-memory implementations of all store traits.
//! These implementations are primarily used for testing and development.
//!
//! # Architecture
//!
//! Store implementations are organized by service:
//! - `InMemoryWamiStore` - Multi-cloud IAM (identity + credentials + policies)
//! - `InMemoryStsStore` - Security Token Service (sessions + identities)
//! - `InMemorySsoAdminStore` - SSO Administration (permission sets, assignments, etc.)
//! - `InMemoryTenantStore` - Tenant management
//! - `InMemoryStore` - Combines all stores into a single unified interface

mod sso_admin;
mod sts;
mod tenant;
mod unified;
mod wami;

// Sub-directories for sub-trait implementations
mod credentials;
mod identity;
mod policies;
mod reports;

// Store implementations
pub use sso_admin::InMemorySsoAdminStore;
pub use sts::InMemoryStsStore;
pub use tenant::InMemoryTenantStore;
pub use unified::InMemoryStore;
pub use wami::InMemoryWamiStore;
