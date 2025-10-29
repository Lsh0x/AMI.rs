//! MfaDevice Builder

use super::model::MfaDevice;
use crate::provider::{CloudProvider, ProviderConfig, ResourceType};

/// Build a new MfaDevice resource
pub fn build_mfa_device(
    user_name: String,
    serial_number: String,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> MfaDevice {
    let wami_arn =
        provider.generate_wami_arn(ResourceType::MfaDevice, account_id, "/", &serial_number);

    MfaDevice {
        user_name,
        serial_number,
        enable_date: chrono::Utc::now(),
        wami_arn,
        providers: Vec::new(),
    }
}

/// Add a provider configuration to an MfaDevice
pub fn add_provider_to_mfa_device(mut mfa_device: MfaDevice, config: ProviderConfig) -> MfaDevice {
    mfa_device.providers.push(config);
    mfa_device
}
