//! MFA Device Service
//!
//! Orchestrates MFA device management operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::MfaDeviceStore;
use crate::wami::credentials::mfa_device::{
    builder as mfa_builder, EnableMfaDeviceRequest, ListMfaDevicesRequest, MfaDevice,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM MFA devices
///
/// Provides high-level operations for MFA device management.
pub struct MfaDeviceService<S> {
    store: Arc<RwLock<S>>,
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: MfaDeviceStore> MfaDeviceService<S> {
    /// Create a new MfaDeviceService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>, account_id: String) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
            account_id,
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
            account_id: self.account_id.clone(),
        }
    }

    /// Create and enable a new MFA device
    pub async fn create_mfa_device(&self, request: EnableMfaDeviceRequest) -> Result<MfaDevice> {
        // Use wami builder to create MFA device
        let mfa_device = mfa_builder::build_mfa_device(
            request.user_name,
            request.serial_number,
            &*self.provider,
            &self.account_id,
        );

        // Store it
        self.store
            .write()
            .unwrap()
            .create_mfa_device(mfa_device)
            .await
    }

    /// Get an MFA device by serial number
    pub async fn get_mfa_device(&self, serial_number: &str) -> Result<Option<MfaDevice>> {
        self.store
            .read()
            .unwrap()
            .get_mfa_device(serial_number)
            .await
    }

    /// Delete an MFA device
    pub async fn delete_mfa_device(&self, serial_number: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_mfa_device(serial_number)
            .await
    }

    /// List MFA devices for a user
    pub async fn list_mfa_devices(&self, request: ListMfaDevicesRequest) -> Result<Vec<MfaDevice>> {
        self.store
            .read()
            .unwrap()
            .list_mfa_devices(&request.user_name)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> MfaDeviceService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        MfaDeviceService::new(store, "123456789012".to_string())
    }

    #[tokio::test]
    async fn test_create_and_get_mfa_device() {
        let service = setup_service();

        let request = EnableMfaDeviceRequest {
            user_name: "alice".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/alice-device".to_string(),
            authentication_code_1: "123456".to_string(),
            authentication_code_2: "789012".to_string(),
        };

        let mfa_device = service.create_mfa_device(request).await.unwrap();
        assert_eq!(mfa_device.user_name, "alice");

        let retrieved = service
            .get_mfa_device(&mfa_device.serial_number)
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_name, "alice");
    }

    #[tokio::test]
    async fn test_delete_mfa_device() {
        let service = setup_service();

        let request = EnableMfaDeviceRequest {
            user_name: "bob".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/bob-device".to_string(),
            authentication_code_1: "123456".to_string(),
            authentication_code_2: "789012".to_string(),
        };
        let mfa_device = service.create_mfa_device(request).await.unwrap();

        service
            .delete_mfa_device(&mfa_device.serial_number)
            .await
            .unwrap();

        let retrieved = service
            .get_mfa_device(&mfa_device.serial_number)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_mfa_devices() {
        let service = setup_service();

        // Create multiple MFA devices for same user
        for i in 0..3 {
            let request = EnableMfaDeviceRequest {
                user_name: "charlie".to_string(),
                serial_number: format!("arn:aws:iam::123456789012:mfa/charlie-device-{}", i),
                authentication_code_1: "123456".to_string(),
                authentication_code_2: "789012".to_string(),
            };
            service.create_mfa_device(request).await.unwrap();
        }

        let list_request = ListMfaDevicesRequest {
            user_name: "charlie".to_string(),
        };
        let devices = service.list_mfa_devices(list_request).await.unwrap();
        assert_eq!(devices.len(), 3);
    }
}
