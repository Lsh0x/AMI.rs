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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_store_new() {
        let _store = InMemoryStore::new();
        // Just verify it can be created
    }

    #[test]
    fn test_in_memory_store_default() {
        let _store = InMemoryStore::default();
        // Just verify it can be created
    }

    #[tokio::test]
    async fn test_store_methods() {
        let mut store = InMemoryStore::new();

        let _wami_store = store.wami_store().await.unwrap();
        // Just verify it doesn't panic

        let _sts_store = store.sts_store().await.unwrap();

        let _sso_store = store.sso_admin_store().await.unwrap();

        let _tenant_store = store.tenant_store().await.unwrap();
    }
}
