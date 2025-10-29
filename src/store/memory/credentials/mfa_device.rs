//! MFA Device Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::MfaDeviceStore;
use crate::wami::credentials::MfaDevice;
use async_trait::async_trait;

#[async_trait]
impl MfaDeviceStore for InMemoryWamiStore {
    async fn create_mfa_device(&mut self, device: MfaDevice) -> Result<MfaDevice> {
        self.mfa_devices
            .insert(device.serial_number.clone(), device.clone());
        Ok(device)
    }

    async fn get_mfa_device(&self, serial_number: &str) -> Result<Option<MfaDevice>> {
        Ok(self.mfa_devices.get(serial_number).cloned())
    }

    async fn delete_mfa_device(&mut self, serial_number: &str) -> Result<()> {
        self.mfa_devices.remove(serial_number);
        Ok(())
    }

    async fn list_mfa_devices(&self, user_name: &str) -> Result<Vec<MfaDevice>> {
        let devices: Vec<MfaDevice> = self
            .mfa_devices
            .values()
            .filter(|device| device.user_name == user_name)
            .cloned()
            .collect();
        Ok(devices)
    }
}
