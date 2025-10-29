//! STS Store Sub-Traits
//!
//! Defines focused sub-traits for STS (Security Token Service) operations.
//! This is a pure persistence layer - sessions and identities carry their own tenant/account info.
//!
//! # Architecture
//!
//! ```text
//! StsStore (composite)
//!   ├── SessionStore  - Session management (4 methods)
//!   └── IdentityStore - Identity tracking (3 methods)
//! ```

mod identity;
mod session;

pub use identity::IdentityStore;
pub use session::SessionStore;

use async_trait::async_trait;

/// Composite trait combining all STS sub-traits
///
/// This trait is automatically implemented for any type that implements
/// all constituent sub-traits via blanket implementation.
///
/// # Sub-traits
/// - [`SessionStore`] - STS session management
/// - [`IdentityStore`] - Caller identity tracking
#[async_trait]
pub trait StsStore: SessionStore + IdentityStore + Send + Sync {
    // All methods inherited from sub-traits
}

// Blanket implementation: any type implementing all sub-traits gets StsStore for free
impl<T> StsStore for T where T: SessionStore + IdentityStore + Send + Sync {}
