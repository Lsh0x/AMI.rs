//! In-memory STS Store
//!
//! This module contains the STS store struct and its sub-trait implementations:
//! - `session.rs` - SessionStore implementation
//! - `identity.rs` - IdentityStore implementation

use crate::wami::sts::{CallerIdentity, StsSession};
use std::collections::HashMap;

pub mod identity;
pub mod session;

#[cfg(test)]
mod tests;

/// In-memory implementation of STS store
///
/// This is a pure persistence layer that stores sessions and identities for ALL tenants.
/// Each session/identity carries its own tenant_id and account_id.
///
/// # Architecture
///
/// The `StsStore` trait is a composite of sub-traits, each implemented in its own file:
/// - `SessionStore` → `memory/sts/session.rs`
/// - `IdentityStore` → `memory/sts/identity.rs`
#[derive(Debug, Clone, Default)]
pub struct InMemoryStsStore {
    pub(super) sessions: HashMap<String, StsSession>,
    pub(super) identities: HashMap<String, CallerIdentity>,
}

impl InMemoryStsStore {
    /// Create a new empty STS store
    pub fn new() -> Self {
        Self::default()
    }
}

// Note: StsStore is automatically implemented via blanket implementation
// because InMemoryStsStore implements all required sub-traits in other files
