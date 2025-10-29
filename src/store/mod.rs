//! Store Module
//!
//! This module provides the storage abstraction layer for WAMI.
//! It defines the traits for different store types (IAM, STS, SSO Admin, Tenant)
//! and provides implementations (currently in-memory).
//!
//! The store layer is a pure persistence layer with no provider coupling.
//! Resources themselves carry their provider-specific information.

pub mod memory;
pub mod resource;
pub mod traits;

// Re-export traits for convenience
pub use traits::{SsoAdminStore, StsStore, TenantStore, WamiStore};

// Re-export the Store trait
use crate::error::Result;
use async_trait::async_trait;

/// Generic store trait that can be implemented by any backend
///
/// This is a pure persistence layer that stores data for ALL tenants and ALL cloud providers.
/// Resources carry their own provider-specific information (ARNs, account IDs, etc.).
#[async_trait]
pub trait Store: Send + Sync {
    type WamiStore: WamiStore;
    type StsStore: StsStore;
    type SsoAdminStore: SsoAdminStore;
    type TenantStore: TenantStore;

    async fn wami_store(&mut self) -> Result<&mut Self::WamiStore>;
    async fn sts_store(&mut self) -> Result<&mut Self::StsStore>;
    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore>;
    async fn tenant_store(&mut self) -> Result<&mut Self::TenantStore>;
}
