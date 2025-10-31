//! Permission Set Service
//!
//! Orchestrates permission set operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::PermissionSetStore;
use crate::wami::sso_admin::permission_set::PermissionSet;
use std::sync::{Arc, RwLock};

/// Service for managing permission sets
pub struct PermissionSetService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
}

impl<S: PermissionSetStore> PermissionSetService<S> {
    /// Create a new PermissionSetService with default AWS provider
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

    /// Create a new permission set
    pub async fn create_permission_set(
        &self,
        permission_set: PermissionSet,
    ) -> Result<PermissionSet> {
        self.store
            .write()
            .unwrap()
            .create_permission_set(permission_set)
            .await
    }

    /// Get a permission set by ARN
    pub async fn get_permission_set(
        &self,
        permission_set_arn: &str,
    ) -> Result<Option<PermissionSet>> {
        self.store
            .read()
            .unwrap()
            .get_permission_set(permission_set_arn)
            .await
    }

    /// Update a permission set
    pub async fn update_permission_set(
        &self,
        permission_set: PermissionSet,
    ) -> Result<PermissionSet> {
        self.store
            .write()
            .unwrap()
            .update_permission_set(permission_set)
            .await
    }

    /// Delete a permission set
    pub async fn delete_permission_set(&self, permission_set_arn: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_permission_set(permission_set_arn)
            .await
    }

    /// List permission sets for an instance
    pub async fn list_permission_sets(&self, instance_arn: &str) -> Result<Vec<PermissionSet>> {
        self.store
            .read()
            .unwrap()
            .list_permission_sets(instance_arn)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use chrono::Utc;

    fn setup_service() -> PermissionSetService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        PermissionSetService::new(store)
    }

    fn create_test_permission_set(name: &str, instance_arn: &str) -> PermissionSet {
        PermissionSet {
            permission_set_arn: format!("arn:aws:sso:::permissionSet/{}/ps-{}", instance_arn, name),
            name: name.to_string(),
            description: Some(format!("Test permission set {}", name)),
            session_duration: Some("PT8H".to_string()),
            relay_state: None,
            instance_arn: instance_arn.to_string(),
            created_date: Utc::now(),
            wami_arn: format!(
                "arn:wami:sso-admin:root:wami:123456789012:permission-set/ps-{}",
                name
            )
            .parse()
            .unwrap(),
            providers: vec![],
        }
    }

    #[tokio::test]
    async fn test_create_and_get_permission_set() {
        let service = setup_service();
        let permission_set = create_test_permission_set("read-only", "instance-1");

        let created = service
            .create_permission_set(permission_set.clone())
            .await
            .unwrap();
        assert_eq!(created.name, "read-only");

        let retrieved = service
            .get_permission_set(&permission_set.permission_set_arn)
            .await
            .unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_update_permission_set() {
        let service = setup_service();
        let mut permission_set = create_test_permission_set("admin", "instance-1");

        service
            .create_permission_set(permission_set.clone())
            .await
            .unwrap();

        permission_set.description = Some("Updated description".to_string());
        let updated = service.update_permission_set(permission_set).await.unwrap();
        assert_eq!(updated.description, Some("Updated description".to_string()));
    }

    #[tokio::test]
    async fn test_delete_permission_set() {
        let service = setup_service();
        let permission_set = create_test_permission_set("temp", "instance-1");

        service
            .create_permission_set(permission_set.clone())
            .await
            .unwrap();
        service
            .delete_permission_set(&permission_set.permission_set_arn)
            .await
            .unwrap();

        let retrieved = service
            .get_permission_set(&permission_set.permission_set_arn)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_permission_sets() {
        let service = setup_service();
        let instance_arn = "instance-1";

        service
            .create_permission_set(create_test_permission_set("ps1", instance_arn))
            .await
            .unwrap();
        service
            .create_permission_set(create_test_permission_set("ps2", instance_arn))
            .await
            .unwrap();

        let permission_sets = service.list_permission_sets(instance_arn).await.unwrap();
        assert_eq!(permission_sets.len(), 2);
    }
}
