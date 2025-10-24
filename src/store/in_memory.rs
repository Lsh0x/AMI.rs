use crate::error::Result;
use crate::store::{IamStore, StsStore, SsoAdminStore, Store};
use crate::store::memory::{InMemoryIamStore, InMemoryStsStore, InMemorySsoAdminStore};

/// Main store implementation that combines all sub-stores
#[derive(Debug)]
pub struct InMemoryStore {
    pub account_id: String,
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
        let account_id = crate::types::AwsConfig::generate_account_id();
        log::info!("Generated AWS account ID: {}", account_id);
        Self {
            account_id: account_id.clone(),
            iam_store: InMemoryIamStore::with_account_id(account_id.clone()),
            sts_store: InMemoryStsStore::with_account_id(account_id.clone()),
            sso_admin_store: InMemorySsoAdminStore::default(),
        }
    }
    
    pub fn with_account_id(account_id: String) -> Self {
        log::info!("Using provided AWS account ID: {}", account_id);
        Self {
            account_id: account_id.clone(),
            iam_store: InMemoryIamStore::with_account_id(account_id.clone()),
            sts_store: InMemoryStsStore::with_account_id(account_id.clone()),
            sso_admin_store: InMemorySsoAdminStore::default(),
        }
    }
    
    /// Get the current AWS account ID
    pub fn account_id(&self) -> &str {
        &self.account_id
    }
    
    /// Get the current AWS account ID as a String
    pub fn account_id_string(&self) -> String {
        self.account_id.clone()
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
