//! SSO Instance Service
//!
//! Orchestrates SSO instance operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::SsoInstanceStore;
use crate::wami::sso_admin::instance::SsoInstance;
use std::sync::{Arc, RwLock};

/// Service for managing SSO instances
pub struct InstanceService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
}

impl<S: SsoInstanceStore> InstanceService<S> {
    /// Create a new InstanceService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
        }
    }

    /// Create a new SSO instance
    pub async fn create_instance(&self, instance: SsoInstance) -> Result<SsoInstance> {
        self.store.write().unwrap().create_instance(instance).await
    }

    /// Get an SSO instance by ARN
    pub async fn get_instance(&self, instance_arn: &str) -> Result<Option<SsoInstance>> {
        self.store.read().unwrap().get_instance(instance_arn).await
    }

    /// List all SSO instances
    pub async fn list_instances(&self) -> Result<Vec<SsoInstance>> {
        self.store.read().unwrap().list_instances().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use chrono::Utc;

    fn setup_service() -> InstanceService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        InstanceService::new(store)
    }

    fn create_test_instance(name: &str) -> SsoInstance {
        SsoInstance {
            instance_arn: format!("arn:aws:sso:::instance/{}", name),
            identity_store_id: format!("d-{}", name),
            name: Some(name.to_string()),
            status: "ACTIVE".to_string(),
            created_date: Utc::now(),
            wami_arn: format!(
                "arn:wami:sso-admin:root:wami:123456789012:instance/{}",
                name
            )
            .parse()
            .unwrap(),
            providers: vec![],
        }
    }

    #[tokio::test]
    async fn test_create_and_get_instance() {
        let service = setup_service();
        let instance = create_test_instance("test-instance");

        let created = service.create_instance(instance.clone()).await.unwrap();
        assert_eq!(created.name, Some("test-instance".to_string()));

        let retrieved = service.get_instance(&instance.instance_arn).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, Some("test-instance".to_string()));
    }

    #[tokio::test]
    async fn test_list_instances() {
        let service = setup_service();

        service
            .create_instance(create_test_instance("instance1"))
            .await
            .unwrap();
        service
            .create_instance(create_test_instance("instance2"))
            .await
            .unwrap();

        let instances = service.list_instances().await.unwrap();
        assert_eq!(instances.len(), 2);
    }
}
