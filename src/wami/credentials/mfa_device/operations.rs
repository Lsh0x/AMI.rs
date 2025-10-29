//! MFA Device Domain Operations
//!
//! Pure business logic functions for MFA device management.

use super::{builder, model::MfaDevice, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::wami::tenant::TenantId;

/// Pure domain operations for MFA devices
pub mod mfa_device_operations {
    use super::*;

    /// Build a new MFA device (pure function)
    pub fn build_from_request(
        request: EnableMfaDeviceRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> MfaDevice {
        builder::build_mfa_device(
            request.user_name,
            request.serial_number,
            provider,
            account_id,
        )
    }

    /// Check if MFA device belongs to tenant (pure predicate)
    pub fn belongs_to_tenant(device: &MfaDevice, tenant_id: &TenantId) -> bool {
        device.tenant_id.as_ref() == Some(tenant_id)
    }

    /// Filter MFA devices by tenant (pure function)
    pub fn filter_by_tenant(devices: Vec<MfaDevice>, tenant_id: &TenantId) -> Vec<MfaDevice> {
        devices
            .into_iter()
            .filter(|d| belongs_to_tenant(d, tenant_id))
            .collect()
    }

    /// Validate MFA device exists and belongs to tenant
    pub fn validate_mfa_device_access(
        device: Option<MfaDevice>,
        serial_number: &str,
        tenant_id: &TenantId,
    ) -> Result<MfaDevice> {
        match device {
            Some(d) if belongs_to_tenant(&d, tenant_id) => Ok(d),
            Some(_) => Err(AmiError::AccessDenied { message: format!("
                resource: format!("MfaDevice: {}", serial_number),
                reason: "MFA device does not belong to current tenant".to_string(),
            }),
            None => Err(AmiError::ResourceNotFound {
                resource: format!("MfaDevice: {}", serial_number),
            }),
        }
    }
}
