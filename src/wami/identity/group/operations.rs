//! Group Domain Operations
//!
//! Pure business logic functions for group management.

use super::{builder, model::Group, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::types::Tag;
use crate::wami::tenant::TenantId;

/// Pure domain operations for groups
pub mod group_operations {
    use super::*;

    /// Build a new group from a request (pure function)
    pub fn build_from_request(
        request: CreateGroupRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> Group {
        builder::build_group(
            request.group_name,
            request.path,
            request.tags,
            provider,
            account_id,
        )
    }

    /// Apply an update to an existing group (pure function)
    pub fn apply_update(
        existing_group: Group,
        request: UpdateGroupRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> Group {
        builder::update_group(
            existing_group,
            request.new_group_name,
            request.new_path,
            provider,
            account_id,
        )
    }

    /// Check if group belongs to tenant (pure predicate)
    pub fn belongs_to_tenant(group: &Group, tenant_id: &TenantId) -> bool {
        group.tenant_id.as_ref() == Some(tenant_id)
    }

    /// Filter groups by tenant (pure function)
    pub fn filter_by_tenant(groups: Vec<Group>, tenant_id: &TenantId) -> Vec<Group> {
        groups
            .into_iter()
            .filter(|g| belongs_to_tenant(g, tenant_id))
            .collect()
    }

    /// Validate group exists and belongs to tenant
    pub fn validate_group_access(
        group: Option<Group>,
        group_name: &str,
        tenant_id: &TenantId,
    ) -> Result<Group> {
        match group {
            Some(g) if belongs_to_tenant(&g, tenant_id) => Ok(g),
            Some(_) => Err(AmiError::AccessDenied { message: format!("
                resource: format!("Group: {}", group_name),
                reason: "Group does not belong to current tenant".to_string(),
            }),
            None => Err(AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            }),
        }
    }
}
