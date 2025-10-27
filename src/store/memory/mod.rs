//! In-Memory Store Implementations
//!
//! This module contains in-memory implementations of all store traits.
//! These implementations are primarily used for testing and development.

mod iam;
mod sso_admin;
mod sts;
mod tenant;
mod unified;

pub use iam::InMemoryIamStore;
pub use sso_admin::InMemorySsoAdminStore;
pub use sts::InMemoryStsStore;
pub use tenant::InMemoryTenantStore;
pub use unified::InMemoryStore;
