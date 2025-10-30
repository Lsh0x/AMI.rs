//! WAMI Store Trait
//!
//! Composite trait combining all IAM-related sub-traits for WAMI (Multi-cloud IAM).
//!
//! # Architecture
//!
//! This trait uses the Interface Segregation Principle to compose focused sub-traits:
//! - **Identity**: `UserStore`, `GroupStore`, `RoleStore`, `ServiceLinkedRoleStore`
//! - **Credentials**: `AccessKeyStore`, `MfaDeviceStore`, `LoginProfileStore`, `ServerCertificateStore`, `SigningCertificateStore`, `ServiceCredentialStore`
//! - **Policies**: `PolicyStore`
//! - **Reports**: `CredentialReportStore`
//!
//! # Flexibility
//!
//! You can implement:
//! 1. **Full Store**: Implement all sub-traits to get `WamiStore` automatically via blanket impl
//! 2. **Partial Store**: Implement only the sub-traits you need (e.g., just `UserStore + GroupStore`)
//! 3. **Mixed Backends**: Use different backends for different resource types via a composite store
//!
//! # Example
//!
//! See the `InMemoryWamiStore` for a complete example implementation of all sub-traits.

use super::credentials::{AccessKeyStore, LoginProfileStore, MfaDeviceStore};
use super::identity::{GroupStore, RoleStore, ServiceLinkedRoleStore, UserStore};
use super::policies::PolicyStore;
use super::reports::CredentialReportStore;
use super::sso_admin::{
    AccountAssignmentStore, ApplicationStore, PermissionSetStore, SsoInstanceStore,
    TrustedTokenIssuerStore,
};
use super::sts::{IdentityStore, SessionStore};
use super::tenant::TenantStore;
use async_trait::async_trait;

/// Composite trait for complete WAMI store functionality
///
/// This trait combines all identity sub-traits for multi-cloud IAM operations.
/// It's automatically implemented for any type that implements all the constituent
/// sub-traits (via blanket implementation).
#[async_trait]
pub trait WamiStore:
    // Identity
    UserStore
    + GroupStore
    + RoleStore
    + ServiceLinkedRoleStore
    // Credentials
    + AccessKeyStore
    + MfaDeviceStore
    + LoginProfileStore
    // TODO: Temporarily disabled during refactor
    // + ServerCertificateStore
    // + SigningCertificateStore
    // + ServiceCredentialStore
    // Policies
    + PolicyStore
    // Reports
    + CredentialReportStore
    // STS
    + SessionStore
    + IdentityStore
    // SSO Admin
    + SsoInstanceStore
    + PermissionSetStore
    + AccountAssignmentStore
    + ApplicationStore
    + TrustedTokenIssuerStore
    // Tenant
    + TenantStore
    // Markers
    + Send
    + Sync
{
    // No methods - all inherited from sub-traits!
}

// Blanket implementation: any type implementing all sub-traits gets WamiStore for free
impl<T> WamiStore for T where
    T: UserStore
        + GroupStore
        + RoleStore
        + ServiceLinkedRoleStore
        + AccessKeyStore
        + MfaDeviceStore
        + LoginProfileStore
        // TODO: Temporarily disabled during refactor
        // + ServerCertificateStore
        // + SigningCertificateStore
        // + ServiceCredentialStore
        + PolicyStore
        + CredentialReportStore
        + SessionStore
        + IdentityStore
        + SsoInstanceStore
        + PermissionSetStore
        + AccountAssignmentStore
        + ApplicationStore
        + TrustedTokenIssuerStore
        + TenantStore
        + Send
        + Sync
{
}
