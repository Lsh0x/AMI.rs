//! MfaDevice Builder

use super::model::MfaDevice;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::provider::ProviderConfig;

/// Build a new MfaDevice resource with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_mfa_device(
    user_name: String,
    serial_number: String,
    context: &WamiContext,
) -> Result<MfaDevice> {
    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("mfa", &serial_number)
        .build()?;

    Ok(MfaDevice {
        user_name,
        serial_number,
        enable_date: chrono::Utc::now(),
        wami_arn,
        providers: Vec::new(),
    })
}

/// Add a provider configuration to an MfaDevice
pub fn add_provider_to_mfa_device(mut mfa_device: MfaDevice, config: ProviderConfig) -> MfaDevice {
    mfa_device.providers.push(config);
    mfa_device
}
