//! In-memory WAMI Store
//!
//! This is the main struct for the WAMI store. The actual trait implementations
//! are split across multiple files organized by functionality:
//! - `identity/` - UserStore, GroupStore, RoleStore
//! - `credentials/` - AccessKeyStore, MfaDeviceStore, LoginProfileStore
//! - `policies/` - PolicyStore

use crate::wami::credentials::{AccessKey, LoginProfile, MfaDevice};
use crate::wami::identity::identity_provider::{OidcProvider, SamlProvider};
use crate::wami::identity::{Group, Role, User};
use crate::wami::policies::Policy;
use crate::wami::sso_admin::{
    AccountAssignment, Application, PermissionSet, SsoInstance, TrustedTokenIssuer,
};
use crate::wami::sts::{CallerIdentity, StsSession};
use crate::wami::tenant::{Tenant, TenantId};
use std::collections::HashMap;

/// In-memory implementation of WAMI store
///
/// This is a pure persistence layer that stores identity resources for ALL tenants
/// across ALL cloud providers. Each resource carries its own tenant_id, account_id,
/// and provider information.
///
/// # Architecture
///
/// The `WamiStore` trait is a composite of multiple sub-traits, each implemented
/// in its own file:
/// - `UserStore` → `memory/identity/user.rs`
/// - `GroupStore` → `memory/identity/group.rs`
/// - `RoleStore` → `memory/identity/role.rs`
/// - `PolicyStore` → `memory/policies/policy.rs` (TODO)
/// - `AccessKeyStore` → `memory/credentials/access_key.rs`
/// - `MfaDeviceStore` → `memory/credentials/mfa_device.rs` (TODO)
/// - `LoginProfileStore` → `memory/credentials/login_profile.rs` (TODO)
#[derive(Debug, Clone, Default)]
pub struct InMemoryWamiStore {
    pub(super) users: HashMap<String, User>,
    pub(super) access_keys: HashMap<String, AccessKey>,
    pub(super) groups: HashMap<String, Group>,
    pub(super) roles: HashMap<String, Role>,
    pub(super) policies: HashMap<String, Policy>,
    pub(super) mfa_devices: HashMap<String, MfaDevice>,
    pub(super) login_profiles: HashMap<String, LoginProfile>,
    pub(super) user_groups: HashMap<String, Vec<String>>, // user_name -> group_names
    pub(super) credential_report: Option<crate::wami::reports::credential_report::CredentialReport>,
    #[allow(dead_code)]
    pub(super) server_certificates: HashMap<String, crate::wami::credentials::ServerCertificate>,
    #[allow(dead_code)]
    pub(super) service_specific_credentials:
        HashMap<String, crate::wami::credentials::service_credential::ServiceSpecificCredential>,
    pub(super) service_linked_role_deletion_tasks:
        HashMap<String, crate::wami::identity::service_linked_role::DeletionTaskInfo>,
    #[allow(dead_code)]
    pub(super) signing_certificates:
        HashMap<String, crate::wami::credentials::signing_certificate::SigningCertificate>,
    // STS resources
    pub(super) sessions: HashMap<String, StsSession>,
    pub(super) identities: HashMap<String, CallerIdentity>,
    // Tenant resources
    pub(super) tenants: HashMap<TenantId, Tenant>,
    // SSO Admin resources
    pub(super) sso_instances: HashMap<String, SsoInstance>,
    pub(super) permission_sets: HashMap<String, PermissionSet>,
    pub(super) account_assignments: HashMap<String, AccountAssignment>,
    pub(super) applications: HashMap<String, Application>,
    pub(super) trusted_token_issuers: HashMap<String, TrustedTokenIssuer>,
    // Identity Provider resources
    pub(super) saml_providers: HashMap<String, SamlProvider>,
    pub(super) oidc_providers: HashMap<String, OidcProvider>,
}

impl InMemoryWamiStore {
    /// Create a new empty WAMI store
    pub fn new() -> Self {
        Self::default()
    }
}

// Note: WamiStore is automatically implemented via blanket implementation
// because InMemoryWamiStore implements all required sub-traits in other files:
// - UserStore, GroupStore, RoleStore (identity/)
// - AccessKeyStore, MfaDeviceStore, LoginProfileStore (credentials/)
// - PolicyStore (policies/)
