//! Store Module
//!
//! This module provides the storage abstraction layer for WAMI.
//! It defines the traits for different store types (IAM, STS, SSO Admin, Tenant)
//! and provides implementations (currently in-memory).

pub mod memory;
pub mod traits;

// Re-export traits for convenience
pub use traits::{IamStore, SsoAdminStore, StsStore};

// Re-export the Store trait
use crate::error::Result;
use crate::provider::CloudProvider;
use crate::tenant::store::TenantStore;
use async_trait::async_trait;

/// Generic store trait that can be implemented by any backend
#[async_trait]
pub trait Store: Send + Sync {
    type IamStore: IamStore;
    type StsStore: StsStore;
    type SsoAdminStore: SsoAdminStore;
    type TenantStore: TenantStore;

    /// Get the cloud provider for this store
    fn cloud_provider(&self) -> &dyn CloudProvider;

    async fn iam_store(&mut self) -> Result<&mut Self::IamStore>;
    async fn sts_store(&mut self) -> Result<&mut Self::StsStore>;
    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore>;
    async fn tenant_store(&mut self) -> Result<&mut Self::TenantStore>;
}
