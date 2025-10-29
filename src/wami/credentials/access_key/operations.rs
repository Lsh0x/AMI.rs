//! Access Key Domain Operations
//!
//! Pure business logic functions for access key management.

use super::{builder, model::AccessKey, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::wami::tenant::TenantId;

/// Pure domain operations for access keys
pub mod access_key_operations {
    use super::*;

    /// Build a new access key from a request (pure function)
    pub fn build_from_request(
        request: CreateAccessKeyRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> AccessKey {
        builder::build_access_key(
            request.user_name,
            provider,
            account_id,
        )
    }

    /// Apply a status update to an existing access key (pure function)
    pub fn apply_status_update(
        existing_key: AccessKey,
        new_status: String,
    ) -> AccessKey {
        builder::update_access_key_status(existing_key, new_status)
    }

    /// Check if access key belongs to tenant (pure predicate)
    pub fn belongs_to_tenant(key: &AccessKey, tenant_id: &TenantId) -> bool {
        key.tenant_id.as_ref() == Some(tenant_id)
    }

    /// Filter access keys by tenant (pure function)
    pub fn filter_by_tenant(keys: Vec<AccessKey>, tenant_id: &TenantId) -> Vec<AccessKey> {
        keys
            .into_iter()
            .filter(|k| belongs_to_tenant(k, tenant_id))
            .collect()
    }

    /// Validate access key exists and belongs to tenant
    pub fn validate_access_key_access(
        key: Option<AccessKey>,
        access_key_id: &str,
        tenant_id: &TenantId,
    ) -> Result<AccessKey> {
        match key {
            Some(k) if belongs_to_tenant(&k, tenant_id) => Ok(k),
            Some(_) => Err(AmiError::AccessDenied { message: format!("
                resource: format!("AccessKey: {}", access_key_id),
                reason: "Access key does not belong to current tenant".to_string(),
            }),
            None => Err(AmiError::ResourceNotFound {
                resource: format!("AccessKey: {}", access_key_id),
            }),
        }
    }
}
