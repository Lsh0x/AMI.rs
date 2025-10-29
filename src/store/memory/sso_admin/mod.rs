//! In-memory SSO Admin Store
//!
//! This module contains the SSO Admin store struct and its sub-trait implementations:
//! - `permission_set.rs` - PermissionSetStore implementation
//! - `account_assignment.rs` - AccountAssignmentStore implementation
//! - `instance.rs` - SsoInstanceStore implementation
//! - `application.rs` - ApplicationStore implementation
//! - `trusted_token_issuer.rs` - TrustedTokenIssuerStore implementation

use crate::wami::sso_admin::{
    AccountAssignment, Application, PermissionSet, SsoInstance, TrustedTokenIssuer,
};
use std::collections::HashMap;

pub mod account_assignment;
pub mod application;
pub mod instance;
pub mod permission_set;
pub mod trusted_token_issuer;

/// In-memory implementation of SSO Admin store
///
/// # Architecture
///
/// The `SsoAdminStore` trait is a composite of sub-traits, each implemented in its own file:
/// - `PermissionSetStore` → `memory/sso_admin/permission_set.rs`
/// - `AccountAssignmentStore` → `memory/sso_admin/account_assignment.rs`
/// - `SsoInstanceStore` → `memory/sso_admin/instance.rs`
/// - `ApplicationStore` → `memory/sso_admin/application.rs`
/// - `TrustedTokenIssuerStore` → `memory/sso_admin/trusted_token_issuer.rs`
#[derive(Debug, Default, Clone)]
pub struct InMemorySsoAdminStore {
    pub(super) permission_sets: HashMap<String, PermissionSet>,
    pub(super) account_assignments: HashMap<String, AccountAssignment>,
    pub(super) instances: HashMap<String, SsoInstance>,
    pub(super) applications: HashMap<String, Application>,
    pub(super) trusted_token_issuers: HashMap<String, TrustedTokenIssuer>,
}

// Note: SsoAdminStore is automatically implemented via blanket implementation
// because InMemorySsoAdminStore implements all required sub-traits in other files
