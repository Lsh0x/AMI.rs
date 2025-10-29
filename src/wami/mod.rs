//! # WAMI - Web-compatible Access Management Interface
//!
//! A unified, multi-cloud identity and access management system.
//!
//! ## Structure
//!
//! This module is organized by **functionality**, matching the store architecture:
//!
//! - **`identity/`** - Users, Groups, Roles, Service-Linked Roles
//! - **`credentials/`** - Access Keys, MFA Devices, Login Profiles, Certificates
//! - **`policies/`** - IAM Policies and policy evaluation
//! - **`reports/`** - Credential reports and auditing
//! - **`sts/`** - Security Token Service (sessions, federation, assume role)
//! - **`sso_admin/`** - Single Sign-On administration
//! - **`tenant/`** - Multi-tenant management and isolation

// ============================================================================
// Functional Modules (matching store structure)
// ============================================================================

/// Identity management: users, groups, roles, identity providers
pub mod identity {
    pub mod group;
    pub mod identity_provider;
    pub mod role;
    pub mod service_linked_role;
    pub mod user;

    // Re-export types for convenience
    pub use group::Group;
    pub use role::Role;
    pub use service_linked_role::DeletionTaskInfo;
    pub use user::User;
}

/// Credential management: access keys, MFA devices, certificates
pub mod credentials {
    pub mod access_key;
    pub mod login_profile;
    pub mod mfa_device;
    pub mod server_certificate;
    pub mod service_credential;
    pub mod signing_certificate;

    // Re-export types for convenience
    pub use access_key::AccessKey;
    pub use login_profile::LoginProfile;
    pub use mfa_device::MfaDevice;
    pub use server_certificate::ServerCertificate;
    pub use service_credential::ServiceSpecificCredential;
    pub use signing_certificate::SigningCertificate;
}

/// Policy management, evaluation, and permissions boundaries
pub mod policies {
    pub mod evaluation;
    pub mod permissions_boundary;
    pub mod policy;

    // Re-export types for convenience
    pub use policy::Policy;
}

/// Report generation and auditing
pub mod reports {
    pub mod credential_report;

    // Re-export types for convenience
    pub use credential_report::CredentialReport;
}

// ============================================================================
// Service Modules
// ============================================================================

/// Security Token Service (STS) - temporary credentials and federation
pub mod sts;

/// Single Sign-On Administration
pub mod sso_admin;

/// Resource tagging
pub mod tags;

/// Tenant management and multi-tenancy
pub mod tenant;

// Client operations (IAM operations)
// Operations moved to service/ layer
// pub mod operations;
// pub mod operations_simple;

// Re-export STS client and types
pub use sts::{Credentials, StsSession};

// Re-export tenant client and types
pub use tenant::{Tenant, TenantId};
// pub use tenant::TenantClient; // TODO: Rebuild in service layer

// ============================================================================
// Legacy IAM Module Compatibility (Temporary)
// ============================================================================

/// Legacy IAM module - to be removed after full migration
#[deprecated(
    since = "0.2.0",
    note = "Use `wami::identity`, `wami::credentials`, etc. instead"
)]
pub mod iam {
    pub use super::credentials::*;
    pub use super::identity::*;
    pub use super::policies::*;
    pub use super::reports::*;
}
