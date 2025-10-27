//! Tenant Client for Managing Tenants

use super::{
    check_tenant_permission, BillingInfo, QuotaMode, Tenant, TenantAction, TenantId, TenantQuotas,
    TenantStatus, TenantType,
};
use crate::error::{AmiError, Result};
use crate::store::traits::TenantStore;
use crate::store::Store;
use crate::types::AmiResponse;
use std::collections::HashMap;

/// Client for tenant management operations
pub struct TenantClient<S: Store> {
    store: S,
    /// Current principal (user ARN) making requests
    current_principal: String,
}

impl<S: Store> TenantClient<S> {
    /// Create a new tenant client
    pub fn new(store: S, principal: String) -> Self {
        Self {
            store,
            current_principal: principal,
        }
    }

    /// Create a root tenant
    pub async fn create_root_tenant(
        &mut self,
        request: CreateRootTenantRequest,
    ) -> Result<AmiResponse<Tenant>> {
        let tenant_id = TenantId::root(&request.name);

        // Generate WAMI ARN for tenant using ARN builder
        let arn_builder = crate::provider::arn_builder::WamiArnBuilder::new();
        let tenant_arn =
            arn_builder.build_arn("tenant", tenant_id.as_str(), "tenant", "/", &request.name);

        let tenant = Tenant {
            id: tenant_id,
            parent_id: None,
            name: request.name,
            organization: request.organization,
            tenant_type: TenantType::Root,
            provider_accounts: request.provider_accounts.unwrap_or_default(),
            arn: tenant_arn,
            providers: Vec::new(), // Providers will be added as they're configured
            created_at: chrono::Utc::now(),
            status: TenantStatus::Active,
            quotas: request.quotas.unwrap_or_default(),
            quota_mode: QuotaMode::Override,
            max_child_depth: request.max_child_depth.unwrap_or(10),
            can_create_sub_tenants: true,
            admin_principals: request.admin_principals,
            metadata: request.metadata.unwrap_or_default(),
            billing_info: request.billing_info,
        };

        let created = self
            .store
            .tenant_store()
            .await?
            .create_tenant(tenant)
            .await?;

        Ok(AmiResponse::success(created))
    }

    /// Create a sub-tenant
    pub async fn create_sub_tenant(
        &mut self,
        parent_id: &TenantId,
        request: CreateSubTenantRequest,
    ) -> Result<AmiResponse<Tenant>> {
        // 1. Check permission
        let has_permission = check_tenant_permission(
            &mut self.store,
            &self.current_principal,
            parent_id,
            TenantAction::CreateSubTenant,
        )
        .await?;

        if !has_permission {
            return Err(AmiError::AccessDenied {
                message: "Not authorized to create sub-tenant".to_string(),
            });
        }

        // 2. Get parent tenant
        let parent = self
            .store
            .tenant_store()
            .await?
            .get_tenant(parent_id)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Parent tenant {} not found", parent_id),
            })?;

        // 3. Validate constraints
        if !parent.can_create_sub_tenants {
            return Err(AmiError::InvalidParameter {
                message: "Parent tenant cannot create sub-tenants".to_string(),
            });
        }

        if parent_id.depth() >= parent.max_child_depth {
            return Err(AmiError::InvalidParameter {
                message: "Maximum tenant hierarchy depth exceeded".to_string(),
            });
        }

        // 4. Check sub-tenant quota
        let children = self
            .store
            .tenant_store()
            .await?
            .list_child_tenants(parent_id)
            .await?;

        if children.len() >= parent.quotas.max_sub_tenants {
            return Err(AmiError::ResourceLimitExceeded {
                resource_type: "sub-tenant".to_string(),
                limit: parent.quotas.max_sub_tenants,
            });
        }

        // 5. Validate child quotas
        if let Some(child_quotas) = &request.quotas {
            child_quotas
                .validate_against_parent(&parent.quotas)
                .map_err(|e| AmiError::InvalidParameter { message: e })?;
        }

        // 6. Create the child tenant
        let child_id = parent_id.child(&request.name);
        let has_custom_quotas = request.quotas.is_some();

        // Generate WAMI ARN for child tenant
        let arn_builder = crate::provider::arn_builder::WamiArnBuilder::new();
        let child_arn =
            arn_builder.build_arn("tenant", child_id.as_str(), "tenant", "/", &request.name);

        let child = Tenant {
            id: child_id,
            parent_id: Some(parent_id.clone()),
            name: request.name,
            organization: request.organization,
            tenant_type: request.tenant_type,
            provider_accounts: request.provider_accounts.unwrap_or_default(),
            arn: child_arn,
            providers: Vec::new(), // Providers will be added as they're configured
            created_at: chrono::Utc::now(),
            status: TenantStatus::Active,
            quotas: request.quotas.unwrap_or_else(|| parent.quotas.clone()),
            quota_mode: if has_custom_quotas {
                QuotaMode::Override
            } else {
                QuotaMode::Inherited
            },
            max_child_depth: parent.max_child_depth.saturating_sub(1),
            can_create_sub_tenants: parent.max_child_depth > 1,
            admin_principals: request.admin_principals,
            metadata: request.metadata.unwrap_or_default(),
            billing_info: request.billing_info,
        };

        let created = self
            .store
            .tenant_store()
            .await?
            .create_tenant(child)
            .await?;

        Ok(AmiResponse::success(created))
    }

    /// Get a tenant by ID
    pub async fn get_tenant(&mut self, tenant_id: &TenantId) -> Result<AmiResponse<Tenant>> {
        let has_permission = check_tenant_permission(
            &mut self.store,
            &self.current_principal,
            tenant_id,
            TenantAction::Read,
        )
        .await?;

        if !has_permission {
            return Err(AmiError::AccessDenied {
                message: "Not authorized to read tenant".to_string(),
            });
        }

        let tenant = self
            .store
            .tenant_store()
            .await?
            .get_tenant(tenant_id)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Tenant {} not found", tenant_id),
            })?;

        Ok(AmiResponse::success(tenant))
    }

    /// List child tenants
    pub async fn list_child_tenants(
        &mut self,
        parent_id: &TenantId,
    ) -> Result<AmiResponse<Vec<Tenant>>> {
        let has_permission = check_tenant_permission(
            &mut self.store,
            &self.current_principal,
            parent_id,
            TenantAction::Read,
        )
        .await?;

        if !has_permission {
            return Err(AmiError::AccessDenied {
                message: "Not authorized to list child tenants".to_string(),
            });
        }

        let children = self
            .store
            .tenant_store()
            .await?
            .list_child_tenants(parent_id)
            .await?;

        Ok(AmiResponse::success(children))
    }

    /// Delete a tenant and all its descendants
    pub async fn delete_tenant_cascade(&mut self, tenant_id: &TenantId) -> Result<AmiResponse<()>> {
        let has_permission = check_tenant_permission(
            &mut self.store,
            &self.current_principal,
            tenant_id,
            TenantAction::Delete,
        )
        .await?;

        if !has_permission {
            return Err(AmiError::AccessDenied {
                message: "Not authorized to delete tenant".to_string(),
            });
        }

        // Get all descendants
        let mut descendants = self
            .store
            .tenant_store()
            .await?
            .get_descendants(tenant_id)
            .await?;

        // Add the tenant itself
        descendants.push(tenant_id.clone());

        // Sort by depth (deepest first) to delete in correct order
        descendants.sort_by_key(|b| std::cmp::Reverse(b.depth()));

        // Delete all in order
        for desc_id in descendants {
            self.store
                .tenant_store()
                .await?
                .delete_tenant(&desc_id)
                .await?;
        }

        Ok(AmiResponse::success(()))
    }
}

/// Request to create a root tenant
#[derive(Debug, Clone)]
pub struct CreateRootTenantRequest {
    pub name: String,
    pub organization: Option<String>,
    pub provider_accounts: Option<HashMap<String, String>>,
    pub quotas: Option<TenantQuotas>,
    pub max_child_depth: Option<usize>,
    pub admin_principals: Vec<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub billing_info: Option<BillingInfo>,
}

/// Request to create a sub-tenant
#[derive(Debug, Clone)]
pub struct CreateSubTenantRequest {
    pub name: String,
    pub organization: Option<String>,
    pub tenant_type: TenantType,
    pub provider_accounts: Option<HashMap<String, String>>,
    pub quotas: Option<TenantQuotas>,
    pub admin_principals: Vec<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub billing_info: Option<BillingInfo>,
}
