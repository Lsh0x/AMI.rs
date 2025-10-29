//! MFA Device Store Trait
//!
//! Focused trait for MFA device storage operations

use crate::error::Result;
use crate::wami::credentials::MfaDevice;
use async_trait::async_trait;

/// Store trait for IAM MFA device operations
#[async_trait]
pub trait MfaDeviceStore: Send + Sync {
    /// Create a new MFA device
    async fn create_mfa_device(&mut self, device: MfaDevice) -> Result<MfaDevice>;

    /// Get an MFA device by serial number
    async fn get_mfa_device(&self, serial_number: &str) -> Result<Option<MfaDevice>>;

    /// Delete an MFA device
    async fn delete_mfa_device(&mut self, serial_number: &str) -> Result<()>;

    /// List MFA devices for a user
    async fn list_mfa_devices(&self, user_name: &str) -> Result<Vec<MfaDevice>>;
}
