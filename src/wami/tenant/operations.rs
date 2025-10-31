//! Tenant Domain Operations - Pure Functions

use super::model::*;
use crate::error::{AmiError, Result};

/// Pure domain operations for tenants
pub mod tenant_operations {
    use super::*;

    /// Build a new tenant (pure function)
    ///
    /// The tenant_id must be pre-generated and guaranteed to be unique.
    /// Use `generate_unique_tenant_id()` in the service layer to ensure uniqueness.
    pub fn build_tenant(
        tenant_id: TenantId,
        name: String,
        organization: Option<String>,
        parent_id: Option<TenantId>,
    ) -> Tenant {
        Tenant {
            id: tenant_id.clone(),
            name,
            organization,
            parent_id,
            created_at: chrono::Utc::now(),
            tenant_type: TenantType::Enterprise,
            provider_accounts: std::collections::HashMap::new(),
            arn: String::new(), // To be filled by caller
            providers: vec![],
            status: TenantStatus::Active,
            quota_mode: QuotaMode::Inherited,
            max_child_depth: 3,
            can_create_sub_tenants: true,
            admin_principals: vec![],
            metadata: std::collections::HashMap::new(),
            quotas: TenantQuotas::default(),
            billing_info: None,
        }
    }

    /// Validate tenant name format (pure function)
    #[allow(clippy::result_large_err)]
    pub fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Tenant name cannot be empty".to_string(),
            });
        }

        if name.len() > 64 {
            return Err(AmiError::InvalidParameter {
                message: "Tenant name cannot exceed 64 characters".to_string(),
            });
        }

        // Allow alphanumeric, hyphens, underscores
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AmiError::InvalidParameter {
                message:
                    "Tenant name can only contain alphanumeric characters, hyphens, and underscores"
                        .to_string(),
            });
        }

        Ok(())
    }

    /// Check if tenant hierarchy depth is valid (pure function)
    pub fn is_valid_depth(tenant_id: &TenantId, max_depth: usize) -> bool {
        tenant_id.depth() <= max_depth
    }

    /// Check if tenant can create sub-tenants (pure function)
    pub fn can_create_child(tenant: &Tenant) -> bool {
        tenant.can_create_sub_tenants && tenant.status == TenantStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tenant_operations::*;

    #[test]
    fn test_build_tenant_basic() {
        let tenant_id = TenantId::root();
        let tenant = build_tenant(tenant_id.clone(), "test-tenant".to_string(), None, None);

        assert_eq!(tenant.name, "test-tenant");
        assert_eq!(tenant.id, tenant_id);
        assert!(tenant.organization.is_none());
        assert!(tenant.parent_id.is_none());
        assert_eq!(tenant.tenant_type, TenantType::Enterprise);
        assert_eq!(tenant.status, TenantStatus::Active);
        assert_eq!(tenant.quota_mode, QuotaMode::Inherited);
        assert_eq!(tenant.max_child_depth, 3);
        assert!(tenant.can_create_sub_tenants);
    }

    #[test]
    fn test_build_tenant_with_organization() {
        let tenant_id = TenantId::root();
        let tenant = build_tenant(
            tenant_id,
            "acme".to_string(),
            Some("ACME Corp".to_string()),
            None,
        );

        assert_eq!(tenant.name, "acme");
        assert_eq!(tenant.organization, Some("ACME Corp".to_string()));
    }

    #[test]
    fn test_build_tenant_with_parent() {
        let parent_id = TenantId::root();
        let tenant_id = parent_id.child();
        let tenant = build_tenant(
            tenant_id.clone(),
            "child".to_string(),
            None,
            Some(parent_id.clone()),
        );

        assert_eq!(tenant.name, "child");
        assert_eq!(tenant.parent_id, Some(parent_id));
        assert_eq!(tenant.id, tenant_id);
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("valid-tenant").is_ok());
        assert!(validate_name("tenant_123").is_ok());
        assert!(validate_name("TenantABC").is_ok());
        assert!(validate_name("a").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        let result = validate_name("");
        assert!(result.is_err());
        assert!(matches!(result, Err(AmiError::InvalidParameter { .. })));
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(65);
        let result = validate_name(&long_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        assert!(validate_name("tenant with spaces").is_err());
        assert!(validate_name("tenant@example").is_err());
        assert!(validate_name("tenant.com").is_err());
        assert!(validate_name("tenant/path").is_err());
    }

    #[test]
    fn test_is_valid_depth() {
        let root = TenantId::root();
        let child = root.child();
        let grandchild = child.child();

        assert!(is_valid_depth(&root, 0));
        assert!(is_valid_depth(&root, 5));

        assert!(!is_valid_depth(&child, 0));
        assert!(is_valid_depth(&child, 1));
        assert!(is_valid_depth(&child, 5));

        assert!(!is_valid_depth(&grandchild, 1));
        assert!(is_valid_depth(&grandchild, 2));
        assert!(is_valid_depth(&grandchild, 10));
    }

    #[test]
    fn test_can_create_child_active() {
        let tenant_id = TenantId::root();
        let mut tenant = build_tenant(tenant_id, "test".to_string(), None, None);
        tenant.can_create_sub_tenants = true;
        tenant.status = TenantStatus::Active;

        assert!(can_create_child(&tenant));
    }

    #[test]
    fn test_can_create_child_suspended() {
        let tenant_id = TenantId::root();
        let mut tenant = build_tenant(tenant_id, "test".to_string(), None, None);
        tenant.can_create_sub_tenants = true;
        tenant.status = TenantStatus::Suspended;

        assert!(!can_create_child(&tenant));
    }

    #[test]
    fn test_can_create_child_disabled() {
        let tenant_id = TenantId::root();
        let mut tenant = build_tenant(tenant_id, "test".to_string(), None, None);
        tenant.can_create_sub_tenants = false;
        tenant.status = TenantStatus::Active;

        assert!(!can_create_child(&tenant));
    }

    #[test]
    fn test_tenant_defaults() {
        let tenant_id = TenantId::root();
        let tenant = build_tenant(tenant_id, "test".to_string(), None, None);

        assert!(tenant.provider_accounts.is_empty());
        assert!(tenant.providers.is_empty());
        assert!(tenant.admin_principals.is_empty());
        assert!(tenant.metadata.is_empty());
        assert!(tenant.billing_info.is_none());
        assert_eq!(tenant.arn, "");
    }
}
