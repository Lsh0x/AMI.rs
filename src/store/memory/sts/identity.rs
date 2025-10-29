//! Identity Store Implementation for InMemoryStsStore

use crate::error::Result;
use crate::store::memory::sts::InMemoryStsStore;
use crate::store::traits::IdentityStore;
use crate::wami::sts::CallerIdentity;
use async_trait::async_trait;

#[async_trait]
impl IdentityStore for InMemoryStsStore {
    async fn create_identity(&mut self, identity: CallerIdentity) -> Result<CallerIdentity> {
        self.identities
            .insert(identity.arn.clone(), identity.clone());
        Ok(identity)
    }

    async fn get_identity(&self, arn: &str) -> Result<Option<CallerIdentity>> {
        Ok(self.identities.get(arn).cloned())
    }

    async fn list_identities(&self) -> Result<Vec<CallerIdentity>> {
        Ok(self.identities.values().cloned().collect())
    }
}
