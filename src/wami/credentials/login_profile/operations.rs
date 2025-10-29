//! Login Profile Domain Operations
//!
//! Pure business logic functions for login profile management.

use super::{builder, model::LoginProfile, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::wami::tenant::TenantId;

/// Pure domain operations for login profiles
pub mod login_profile_operations {
    use super::*;

    /// Build a new login profile (pure function)
    pub fn build_from_request(
        request: CreateLoginProfileRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> LoginProfile {
        builder::build_login_profile(
            request.user_name,
            request.password_reset_required,
            provider,
            account_id,
        )
    }

    /// Apply an update to an existing login profile (pure function)
    pub fn apply_update(
        existing_profile: LoginProfile,
        request: UpdateLoginProfileRequest,
    ) -> LoginProfile {
        builder::update_login_profile(
            existing_profile,
            request.password_reset_required,
        )
    }

    /// Check if login profile belongs to tenant (pure predicate)
    pub fn belongs_to_tenant(profile: &LoginProfile, tenant_id: &TenantId) -> bool {
        profile.tenant_id.as_ref() == Some(tenant_id)
    }

    /// Validate login profile exists and belongs to tenant
    pub fn validate_login_profile_access(
        profile: Option<LoginProfile>,
        user_name: &str,
        tenant_id: &TenantId,
    ) -> Result<LoginProfile> {
        match profile {
            Some(p) if belongs_to_tenant(&p, tenant_id) => Ok(p),
            Some(_) => Err(AmiError::AccessDenied { message: format!("
                resource: format!("LoginProfile for user: {}", user_name),
                reason: "Login profile does not belong to current tenant".to_string(),
            }),
            None => Err(AmiError::ResourceNotFound {
                resource: format!("LoginProfile for user: {}", user_name),
            }),
        }
    }
}
