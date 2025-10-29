//! SSO Admin Store Sub-Traits
//!
//! Defines focused sub-traits for SSO Admin operations.
//!
//! # Architecture
//!
//! ```text
//! SsoAdminStore (composite)
//!   ├── PermissionSetStore      - Permission set management (5 methods)
//!   ├── AccountAssignmentStore  - Account assignments (4 methods)
//!   ├── SsoInstanceStore        - SSO instances (3 methods)
//!   ├── ApplicationStore        - Applications (3 methods)
//!   └── TrustedTokenIssuerStore - Token issuers (4 methods)
//! ```

mod account_assignment;
mod application;
mod instance;
mod permission_set;
mod trusted_token_issuer;

pub use account_assignment::AccountAssignmentStore;
pub use application::ApplicationStore;
pub use instance::SsoInstanceStore;
pub use permission_set::PermissionSetStore;
pub use trusted_token_issuer::TrustedTokenIssuerStore;

use async_trait::async_trait;

/// Composite trait combining all SSO Admin sub-traits
///
/// This trait is automatically implemented for any type that implements
/// all constituent sub-traits via blanket implementation.
///
/// # Sub-traits
/// - [`PermissionSetStore`] - Permission set management
/// - [`AccountAssignmentStore`] - Account assignments
/// - [`SsoInstanceStore`] - SSO instance management
/// - [`ApplicationStore`] - Application management
/// - [`TrustedTokenIssuerStore`] - Trusted token issuer management
#[async_trait]
pub trait SsoAdminStore:
    PermissionSetStore
    + AccountAssignmentStore
    + SsoInstanceStore
    + ApplicationStore
    + TrustedTokenIssuerStore
    + Send
    + Sync
{
    // All methods inherited from sub-traits
}

// Blanket implementation: any type implementing all sub-traits gets SsoAdminStore for free
impl<T> SsoAdminStore for T where
    T: PermissionSetStore
        + AccountAssignmentStore
        + SsoInstanceStore
        + ApplicationStore
        + TrustedTokenIssuerStore
        + Send
        + Sync
{
}
