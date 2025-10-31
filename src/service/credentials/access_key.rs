//! Access Key Service
//!
//! Orchestrates access key management operations.

use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::AccessKeyStore;
use crate::wami::credentials::access_key::{
    builder as access_key_builder, AccessKey, CreateAccessKeyRequest, ListAccessKeysRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM access keys
///
/// Provides high-level operations for access key management.
pub struct AccessKeyService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: AccessKeyStore> AccessKeyService<S> {
    /// Create a new AccessKeyService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Create a new access key
    pub async fn create_access_key(
        &self,
        context: &WamiContext,
        request: CreateAccessKeyRequest,
    ) -> Result<AccessKey> {
        // Use wami builder to create access key
        let access_key = access_key_builder::build_access_key(request.user_name, context)?;

        // Store it
        self.store
            .write()
            .unwrap()
            .create_access_key(access_key)
            .await
    }

    /// Get an access key by ID
    pub async fn get_access_key(&self, access_key_id: &str) -> Result<Option<AccessKey>> {
        self.store
            .read()
            .unwrap()
            .get_access_key(access_key_id)
            .await
    }

    /// Update an access key (e.g., change status)
    pub async fn update_access_key(&self, access_key: AccessKey) -> Result<AccessKey> {
        self.store
            .write()
            .unwrap()
            .update_access_key(access_key)
            .await
    }

    /// Delete an access key
    pub async fn delete_access_key(&self, access_key_id: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_access_key(access_key_id)
            .await
    }

    /// List access keys for a user
    pub async fn list_access_keys(
        &self,
        request: ListAccessKeysRequest,
    ) -> Result<(Vec<AccessKey>, bool, Option<String>)> {
        self.store
            .read()
            .unwrap()
            .list_access_keys(&request.user_name, request.pagination.as_ref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> AccessKeyService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        AccessKeyService::new(store)
    }

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:.*:12345678:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get_access_key() {
        let service = setup_service();

        let request = CreateAccessKeyRequest {
            user_name: "alice".to_string(),
        };

        let context = test_context();
        let access_key = service.create_access_key(&context, request).await.unwrap();
        assert_eq!(access_key.user_name, "alice");
        assert!(!access_key.access_key_id.is_empty());
        assert!(access_key.secret_access_key.is_some());

        let retrieved = service
            .get_access_key(&access_key.access_key_id)
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_name, "alice");
    }

    #[tokio::test]
    async fn test_delete_access_key() {
        let service = setup_service();

        let request = CreateAccessKeyRequest {
            user_name: "bob".to_string(),
        };
        let context = test_context();
        let access_key = service.create_access_key(&context, request).await.unwrap();

        service
            .delete_access_key(&access_key.access_key_id)
            .await
            .unwrap();

        let retrieved = service
            .get_access_key(&access_key.access_key_id)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_access_keys() {
        let service = setup_service();

        // Create multiple access keys for same user
        for _ in 0..3 {
            let request = CreateAccessKeyRequest {
                user_name: "charlie".to_string(),
            };
            let context = test_context();
            service.create_access_key(&context, request).await.unwrap();
        }

        let list_request = ListAccessKeysRequest {
            user_name: "charlie".to_string(),
            pagination: None,
        };
        let (keys, _, _) = service.list_access_keys(list_request).await.unwrap();
        assert_eq!(keys.len(), 3);
    }
}
