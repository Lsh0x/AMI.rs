//! Store Trait Definitions
//!
//! This module contains the trait definitions for all store operations.
//! These traits define the interface that storage backends must implement.
//!
//! # Architecture Evolution
//!
//! ## Legacy Traits (Backward Compatibility)
//!
//! The original trait system used separate traits for each service:
//! - `IamStore` - IAM operations (users, roles, policies, groups)
//! - `StsStore` - STS operations (sessions, credentials)
//! - `SsoAdminStore` - SSO Admin operations
//! - `TenantStore` - Multi-tenant operations
//!
//! These traits are still available for backward compatibility.
//!
//! ## New Unified Trait (Recommended)
//!
//! The new `Store` trait provides a simplified ARN-based interface:
//! - Single trait for all resource types
//! - Generic methods: `get()`, `query()`, `put()`, `delete()`
//! - ARN-centric design for security and multi-cloud support
//! - Better tenant isolation through opaque ARN hashing
//!
//! See the `unified` module for detailed documentation.

mod iam;
mod sso_admin;
mod sts;
mod tenant;
mod unified;

// Legacy traits (backward compatibility)
pub use iam::IamStore;
pub use sso_admin::SsoAdminStore;
pub use sts::StsStore;
pub use tenant::TenantStore;

// New unified trait (recommended)
pub use unified::Store;
