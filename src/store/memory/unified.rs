//! Unified In-Memory Store
//!
//! Combines IAM, STS, SSO Admin, and Tenant stores into a single unified store.
//! This is a pure persistence layer with no provider or account coupling.

use crate::error::Result;
use crate::store::memory::{
    InMemorySsoAdminStore, InMemoryStsStore, InMemoryTenantStore, InMemoryWamiStore,
};
use crate::store::Store;
use async_trait::async_trait;

/// Main store implementation that combines all sub-stores
///
/// This is a unified store that holds data for ALL tenants and ALL cloud providers.
/// Resources themselves carry their provider-specific information (ARNs, account IDs, etc.).
#[derive(Debug, Clone, Default)]
pub struct InMemoryStore {
    pub wami_store: InMemoryWamiStore,
    pub sts_store: InMemoryStsStore,
    pub sso_admin_store: InMemorySsoAdminStore,
    pub tenant_store: InMemoryTenantStore,
}

impl InMemoryStore {
    /// Create a new empty unified store
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Store for InMemoryStore {
    type WamiStore = InMemoryWamiStore;
    type StsStore = InMemoryStsStore;
    type SsoAdminStore = InMemorySsoAdminStore;
    type TenantStore = InMemoryTenantStore;

    async fn wami_store(&mut self) -> Result<&mut Self::WamiStore> {
        Ok(&mut self.wami_store)
    }

    async fn sts_store(&mut self) -> Result<&mut Self::StsStore> {
        Ok(&mut self.sts_store)
    }

    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore> {
        Ok(&mut self.sso_admin_store)
    }

    async fn tenant_store(&mut self) -> Result<&mut Self::TenantStore> {
        Ok(&mut self.tenant_store)
    }
}
