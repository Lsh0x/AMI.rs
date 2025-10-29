//! Store Trait Definitions
//!
//! This module contains the trait definitions for all store operations.
//! These traits define the interface that storage backends must implement.
//!
//! # Architecture
//!
//! ## Interface Segregation Principle
//!
//! The store traits follow the Interface Segregation Principle with focused sub-traits:
//! - `UserStore`, `GroupStore`, `RoleStore` - Identity management
//! - `AccessKeyStore`, `MfaDeviceStore`, `LoginProfileStore` - Credential management
//! - `PolicyStore` - Authorization management
//! - `WamiStore` - Composite trait combining all IAM sub-traits
//! - `StsStore`, `SsoAdminStore`, `TenantStore` - Service-specific traits
//!
//! ## Benefits
//!
//! - **Flexibility**: Implement only what you need (e.g., UserStore + GroupStore)
//! - **Testability**: Easier testing with focused mocks
//! - **Modularity**: Better code organization and separation of concerns
//! - **Scalability**: Parallel development without merge conflicts

// Sub-trait directories (organized by functionality)
mod credentials; // Access Keys, MFA Devices, Login Profiles, Certificates, Service Credentials
mod identity; // Users, Groups, Roles, Service-Linked Roles
mod policies; // Policies
mod reports; // Credential Reports

// Composite trait directories
mod sso_admin;
mod sts; // STS store (sessions + identities)
mod wami; // WAMI store (identity + credentials + policies) // SSO Admin store (permission sets + assignments + instances + apps + issuers)

// Supporting trait modules
mod tenant;

// Export sub-traits from identity
pub use identity::{GroupStore, RoleStore, ServiceLinkedRoleStore, UserStore};

// Export sub-traits from credentials
pub use credentials::{
    AccessKeyStore, LoginProfileStore, MfaDeviceStore, ServerCertificateStore,
    ServiceCredentialStore, SigningCertificateStore,
};

// Export sub-traits from policies
pub use policies::PolicyStore;

// Export sub-traits from reports
pub use reports::CredentialReportStore;

// Export composite traits
pub use sso_admin::{
    AccountAssignmentStore, ApplicationStore, PermissionSetStore, SsoAdminStore, SsoInstanceStore,
    TrustedTokenIssuerStore,
};
pub use sts::{IdentityStore, SessionStore, StsStore};
pub use tenant::TenantStore;
pub use wami::WamiStore;
