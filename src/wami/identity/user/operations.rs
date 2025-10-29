//! User Domain Operations
//!
//! Pure business logic functions for user management.
//! No store dependencies - can be used by any persistence layer.

use super::{builder, model::User, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::types::Tag;
use crate::wami::tenant::TenantId;

/// Pure domain operations for users
pub mod user_operations {
    use super::*;

    /// Build a new user from a request (pure function)
    pub fn build_from_request(
        request: CreateUserRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
        tenant_id: Option<TenantId>,
    ) -> User {
        builder::build_user(
            request.user_name,
            request.path,
            request.permissions_boundary,
            request.tags,
            provider,
            account_id,
            tenant_id,
        )
    }

    /// Apply an update to an existing user (pure function)
    pub fn apply_update(
        existing_user: User,
        request: UpdateUserRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> User {
        builder::update_user(
            existing_user,
            request.new_user_name,
            request.new_path,
            provider,
            account_id,
        )
    }

    /// Check if user belongs to tenant (pure predicate)
    pub fn belongs_to_tenant(user: &User, tenant_id: &TenantId) -> bool {
        user.tenant_id.as_ref() == Some(tenant_id)
    }

    /// Filter users by tenant (pure function)
    pub fn filter_by_tenant(users: Vec<User>, tenant_id: &TenantId) -> Vec<User> {
        users
            .into_iter()
            .filter(|u| belongs_to_tenant(u, tenant_id))
            .collect()
    }

    /// Validate user exists and belongs to tenant
    pub fn validate_user_access(
        user: Option<User>,
        user_name: &str,
        tenant_id: &TenantId,
    ) -> Result<User> {
        match user {
            Some(u) if belongs_to_tenant(&u, tenant_id) => Ok(u),
            Some(_) => Err(AmiError::AccessDenied { message: format!("
                resource: format!("User: {}", user_name),
                reason: "User does not belong to current tenant".to_string(),
            }),
            None => Err(AmiError::ResourceNotFound {
                resource: format!("User: {}", user_name),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ProviderConfig;

    #[test]
    fn test_belongs_to_tenant() {
        let tenant_id = TenantId::new("test-tenant");
        let user = User {
            user_name: "alice".to_string(),
            user_id: "123".to_string(),
            arn: "arn:aws:iam::123456789012:user/alice".to_string(),
            path: "/".to_string(),
            create_date: chrono::Utc::now(),
            password_last_used: None,
            permissions_boundary: None,
            tags: vec![],
            wami_arn: "arn:wami:iam::tenant-hash:user/alice".to_string(),
            providers: vec![],
            tenant_id: Some(tenant_id.clone()),
        };

        assert!(user_operations::belongs_to_tenant(&user, &tenant_id));
        
        let other_tenant = TenantId::new("other-tenant");
        assert!(!user_operations::belongs_to_tenant(&user, &other_tenant));
    }

    #[test]
    fn test_filter_by_tenant() {
        let tenant1 = TenantId::new("tenant1");
        let tenant2 = TenantId::new("tenant2");

        let users = vec![
            User {
                user_name: "alice".to_string(),
                user_id: "1".to_string(),
                arn: "arn:1".to_string(),
                path: "/".to_string(),
                create_date: chrono::Utc::now(),
                password_last_used: None,
                permissions_boundary: None,
                tags: vec![],
                wami_arn: "arn:wami:1".to_string(),
                providers: vec![],
                tenant_id: Some(tenant1.clone()),
            },
            User {
                user_name: "bob".to_string(),
                user_id: "2".to_string(),
                arn: "arn:2".to_string(),
                path: "/".to_string(),
                create_date: chrono::Utc::now(),
                password_last_used: None,
                permissions_boundary: None,
                tags: vec![],
                wami_arn: "arn:wami:2".to_string(),
                providers: vec![],
                tenant_id: Some(tenant2.clone()),
            },
        ];

        let filtered = user_operations::filter_by_tenant(users, &tenant1);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].user_name, "alice");
    }
}
