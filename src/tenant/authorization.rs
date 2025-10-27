//! Tenant Authorization Service
//!
//! Handles authorization logic for tenant operations.
//! This is separate from the store layer to maintain separation of concerns:
//! - Store = Pure persistence
//! - Authorization = Business logic

use crate::error::Result;
use crate::store::traits::TenantStore;
use crate::store::Store;
use crate::tenant::TenantId;

/// Tenant actions for permission checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantAction {
    /// Read tenant information
    Read,
    /// Update tenant
    Update,
    /// Delete tenant
    Delete,
    /// Create sub-tenant
    CreateSubTenant,
    /// Manage users in tenant
    ManageUsers,
    /// Manage roles in tenant
    ManageRoles,
    /// Manage policies in tenant
    ManagePolicies,
}

/// Check if a user has permission to perform an action on a tenant
///
/// Authorization logic:
/// - User is admin of the tenant → allowed
/// - User is admin of any parent tenant → allowed (hierarchical permissions)
/// - Otherwise → denied
///
/// This is a standalone function to keep the authorization logic separate from the store.
pub async fn check_tenant_permission<S: Store>(
    store: &mut S,
    user_arn: &str,
    tenant_id: &TenantId,
    _action: TenantAction,
) -> Result<bool> {
    // Check if user is admin of this tenant
    let tenant_store = store.tenant_store().await?;

    if let Some(tenant) = tenant_store.get_tenant(tenant_id).await? {
        if tenant.admin_principals.contains(&user_arn.to_string()) {
            return Ok(true);
        }
    }

    // Check if user is admin of any parent tenant (hierarchical permissions)
    let ancestors = tenant_store.get_ancestors(tenant_id).await?;
    for ancestor in ancestors {
        if ancestor.admin_principals.contains(&user_arn.to_string()) {
            return Ok(true);
        }
    }

    Ok(false)
}

