//! Role Domain Operations
//!
//! Pure business logic functions for role management.

use super::{builder, model::Role, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::types::{PolicyDocument, Tag};
use crate::wami::tenant::TenantId;

/// Pure domain operations for roles
pub mod role_operations {
    use super::*;

    /// Build a new role from a request (pure function)
    pub fn build_from_request(
        request: CreateRoleRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
        tenant_id: Option<TenantId>,
    ) -> Role {
        builder::build_role(
            request.role_name,
            request.assume_role_policy_document,
            request.path,
            request.description,
            request.max_session_duration,
            request.permissions_boundary,
            request.tags,
            provider,
            account_id,
            tenant_id,
        )
    }

    /// Apply an update to an existing role (pure function)
    pub fn apply_update(
        existing_role: Role,
        description: Option<String>,
        max_session_duration: Option<i32>,
    ) -> Role {
        builder::update_role(existing_role, description, max_session_duration)
    }

    /// Check if role belongs to tenant (pure predicate)
    pub fn belongs_to_tenant(role: &Role, tenant_id: &TenantId) -> bool {
        role.tenant_id.as_ref() == Some(tenant_id)
    }

    /// Filter roles by tenant (pure function)
    pub fn filter_by_tenant(roles: Vec<Role>, tenant_id: &TenantId) -> Vec<Role> {
        roles
            .into_iter()
            .filter(|r| belongs_to_tenant(r, tenant_id))
            .collect()
    }

    /// Validate role exists and belongs to tenant
    pub fn validate_role_access(
        role: Option<Role>,
        role_name: &str,
        tenant_id: &TenantId,
    ) -> Result<Role> {
        match role {
            Some(r) if belongs_to_tenant(&r, tenant_id) => Ok(r),
            Some(_) => Err(AmiError::AccessDenied { message: format!("
                resource: format!("Role: {}", role_name),
                reason: "Role does not belong to current tenant".to_string(),
            }),
            None => Err(AmiError::ResourceNotFound {
                resource: format!("Role: {}", role_name),
            }),
        }
    }
}
