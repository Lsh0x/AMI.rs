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
//! ```rust
//! use wami::store::traits::{UserStore, GroupStore, WamiStore};
//!
//! // Implementing just User and Group operations
//! struct MyPartialStore {
//!     // ... your storage implementation
//! }
//!
//! impl UserStore for MyPartialStore {
//!     // ... implement user methods
//! }
//!
//! impl GroupStore for MyPartialStore {
//!     // ... implement group methods
//! }
//!
//! // Now you can use it with operations that only need users and groups
//! ```

use super::credentials::{AccessKeyStore, LoginProfileStore, MfaDeviceStore};
use super::identity::{GroupStore, RoleStore, ServiceLinkedRoleStore, UserStore};
use super::policies::PolicyStore;
use super::reports::CredentialReportStore;
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
        + Send
        + Sync
{
}
