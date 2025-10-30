//! SSO Instance Store Implementation for InMemorySsoAdminStore

use crate::error::Result;
use crate::store::memory::sso_admin::InMemorySsoAdminStore;
use crate::store::traits::SsoInstanceStore;
use crate::wami::sso_admin::SsoInstance;
use async_trait::async_trait;

#[async_trait]
impl SsoInstanceStore for InMemorySsoAdminStore {
    async fn create_instance(&mut self, instance: SsoInstance) -> Result<SsoInstance> {
        self.instances
            .insert(instance.instance_arn.clone(), instance.clone());
        Ok(instance)
    }

    async fn get_instance(&self, instance_arn: &str) -> Result<Option<SsoInstance>> {
        Ok(self.instances.get(instance_arn).cloned())
    }

    async fn list_instances(&self) -> Result<Vec<SsoInstance>> {
        Ok(self.instances.values().cloned().collect())
    }
}

/// Implement SsoInstanceStore for InMemoryWamiStore (the main unified store)
#[async_trait]
impl SsoInstanceStore for super::super::wami::InMemoryWamiStore {
    async fn create_instance(&mut self, instance: SsoInstance) -> Result<SsoInstance> {
        self.sso_instances
            .insert(instance.instance_arn.clone(), instance.clone());
        Ok(instance)
    }

    async fn get_instance(&self, instance_arn: &str) -> Result<Option<SsoInstance>> {
        Ok(self.sso_instances.get(instance_arn).cloned())
    }

    async fn list_instances(&self) -> Result<Vec<SsoInstance>> {
        Ok(self.sso_instances.values().cloned().collect())
    }
}
