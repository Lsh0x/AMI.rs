use crate::error::Result;
use crate::store::{IamStore, StsStore, SsoAdminStore, Store};
use crate::store::memory::{InMemoryIamStore, InMemoryStsStore, InMemorySsoAdminStore};

/// Main store implementation that combines all sub-stores
#[derive(Debug)]
pub struct InMemoryStore {
    pub iam_store: InMemoryIamStore,
    pub sts_store: InMemoryStsStore,
    pub sso_admin_store: InMemorySsoAdminStore,
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            iam_store: InMemoryIamStore::default(),
            sts_store: InMemoryStsStore::default(),
            sso_admin_store: InMemorySsoAdminStore::default(),
        }
    }
}

#[async_trait]
impl Store for InMemoryStore {
    type IamStore = InMemoryIamStore;
    type StsStore = InMemoryStsStore;
    type SsoAdminStore = InMemorySsoAdminStore;

    async fn iam_store(&mut self) -> Result<&mut Self::IamStore> {
        Ok(&mut self.iam_store)
    }

    async fn sts_store(&mut self) -> Result<&mut Self::StsStore> {
        Ok(&mut self.sts_store)
    }

    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore> {
        Ok(&mut self.sso_admin_store)
    }
}
