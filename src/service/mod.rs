#![allow(clippy::await_holding_lock)]
#![allow(clippy::result_large_err)]
#![allow(clippy::unnecessary_map_or)]
//! Service Layer
//!
//! This layer orchestrates wami pure functions with store persistence.
//! Services provide a high-level API that:
//! - Uses wami builders to create domain objects
//! - Validates and transforms data
//! - Persists to storage via store traits
//!
//! # Architecture
//!
//! ```text
//! Service Layer (orchestration)
//!     ↓ uses
//! Wami Layer (pure functions + builders)
//!     ↓ creates
//! Store Layer (persistence)
//! ```
//!
//! # Structure
//!
//! Services mirror the wami/ and store/ directory structure:
//! - `auth/` - Authentication and Authorization services
//! - `identity/` - User, Group, Role, ServiceLinkedRole services
//! - `credentials/` - AccessKey, MfaDevice, LoginProfile services
//! - `policies/` - Policy service
//! - `reports/` - CredentialReport service
//! - `sts/` - Session, Identity services
//! - `tenant/` - Tenant service

pub mod auth;
pub mod credentials;
pub mod identity;
pub mod policies;
pub mod reports;
pub mod sso_admin;
pub mod sts;
pub mod tenant;

// Re-export main services for convenience
pub use auth::{hash_secret, verify_secret, AuthenticationService, AuthorizationService};
pub use credentials::{
    AccessKeyService, LoginProfileService, MfaDeviceService, ServerCertificateService,
    ServiceCredentialService, SigningCertificateService,
};
pub use identity::{
    GroupService, IdentityProviderService, RoleService, ServiceLinkedRoleService, UserService,
};
pub use policies::{
    AttachmentService, EvaluationService, InlinePolicyService, PermissionsBoundaryService,
    PolicyService,
};
pub use reports::CredentialReportService;
pub use sso_admin::{
    AccountAssignmentService, ApplicationService, InstanceService, PermissionSetService,
    TrustedTokenIssuerService,
};
pub use sts::{
    AssumeRoleService, FederationService, IdentityService, SessionService, SessionTokenService,
};
pub use tenant::TenantService;
