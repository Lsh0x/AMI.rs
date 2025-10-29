//! MfaDevice Request and Response Types

use serde::{Deserialize, Serialize};

/// Request parameters for enabling an MFA device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableMfaDeviceRequest {
    /// The name of the IAM user for whom the MFA device is being enabled
    pub user_name: String,
    /// The serial number that uniquely identifies the MFA device
    pub serial_number: String,
    /// First authentication code from the device
    pub authentication_code_1: String,
    /// Second authentication code from the device
    pub authentication_code_2: String,
}

/// Request parameters for listing MFA devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMfaDevicesRequest {
    /// The name of the user whose MFA devices you want to list
    pub user_name: String,
}
