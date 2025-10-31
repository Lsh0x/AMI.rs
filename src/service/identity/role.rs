//! Role Service
//!
//! Orchestrates role management operations.

use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::RoleStore;
use crate::wami::identity::role::{
    builder as role_builder, CreateRoleRequest, ListRolesRequest, Role, UpdateRoleRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM roles
///
/// Provides high-level operations for role management.
pub struct RoleService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: RoleStore> RoleService<S> {
    /// Create a new RoleService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Create a new role
    pub async fn create_role(
        &self,
        context: &WamiContext,
        request: CreateRoleRequest,
    ) -> Result<Role> {
        // Use wami builder to create role with context
        let mut role = role_builder::build_role(
            request.role_name,
            request.assume_role_policy_document,
            request.path,
            request.description,
            request.max_session_duration,
            context,
        )?;

        // Apply permissions boundary if specified
        if let Some(boundary_arn) = request.permissions_boundary {
            role = role_builder::set_permissions_boundary(role, boundary_arn);
        }

        // Apply tags if specified
        let role = if let Some(tags) = request.tags {
            role_builder::add_tags(role, tags)
        } else {
            role
        };

        // Store it
        self.store.write().unwrap().create_role(role).await
    }

    /// Get a role by name
    pub async fn get_role(&self, role_name: &str) -> Result<Option<Role>> {
        self.store.read().unwrap().get_role(role_name).await
    }

    /// Update a role
    pub async fn update_role(&self, request: UpdateRoleRequest) -> Result<Role> {
        // Get existing role
        let mut role = self
            .store
            .read()
            .unwrap()
            .get_role(&request.role_name)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("Role: {}", request.role_name),
            })?;

        // Apply updates using builder functions
        if let Some(description) = request.description {
            role = role_builder::update_description(role, Some(description));
        }

        if let Some(max_session_duration) = request.max_session_duration {
            role = role_builder::update_max_session_duration(role, max_session_duration);
        }

        // Store updated role
        self.store.write().unwrap().update_role(role).await
    }

    /// Delete a role
    pub async fn delete_role(&self, role_name: &str) -> Result<()> {
        self.store.write().unwrap().delete_role(role_name).await
    }

    /// List roles with optional filtering
    pub async fn list_roles(
        &self,
        request: ListRolesRequest,
    ) -> Result<(Vec<Role>, bool, Option<String>)> {
        self.store
            .read()
            .unwrap()
            .list_roles(request.path_prefix.as_deref(), request.pagination.as_ref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::store::memory::InMemoryWamiStore;
    use crate::types::Tag;

    fn setup_service() -> RoleService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        RoleService::new(store)
    }

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:iam:test:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("test"))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get_role() {
        let service = setup_service();
        let context = test_context();

        let request = CreateRoleRequest {
            role_name: "admin-role".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            path: Some("/admin/".to_string()),
            description: Some("Admin role".to_string()),
            max_session_duration: Some(3600),
            permissions_boundary: None,
            tags: None,
        };

        let role = service.create_role(&context, request).await.unwrap();
        assert_eq!(role.role_name, "admin-role");
        assert_eq!(role.path, "/admin/");
        assert_eq!(role.max_session_duration, Some(3600));

        let retrieved = service.get_role("admin-role").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().role_name, "admin-role");
    }

    #[tokio::test]
    async fn test_update_role() {
        let service = setup_service();

        // Create role
        let create_request = CreateRoleRequest {
            role_name: "test-role".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            path: None,
            description: Some("Test role".to_string()),
            max_session_duration: Some(3600),
            permissions_boundary: None,
            tags: None,
        };
        let context = test_context();
        service.create_role(&context, create_request).await.unwrap();

        // Update role
        let update_request = UpdateRoleRequest {
            role_name: "test-role".to_string(),
            description: Some("Updated description".to_string()),
            max_session_duration: Some(7200),
        };
        let updated = service.update_role(update_request).await.unwrap();
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.max_session_duration, Some(7200));
    }

    #[tokio::test]
    async fn test_delete_role() {
        let service = setup_service();

        let request = CreateRoleRequest {
            role_name: "temp-role".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            path: None,
            description: None,
            max_session_duration: None,
            permissions_boundary: None,
            tags: None,
        };
        let context = test_context();
        service.create_role(&context, request).await.unwrap();

        service.delete_role("temp-role").await.unwrap();

        let retrieved = service.get_role("temp-role").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_roles() {
        let service = setup_service();

        // Create multiple roles
        for name in ["role1", "role2", "role3"] {
            let request = CreateRoleRequest {
                role_name: name.to_string(),
                assume_role_policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#
                    .to_string(),
                path: Some("/test/".to_string()),
                description: None,
                max_session_duration: None,
                permissions_boundary: None,
                tags: None,
            };
            let context = test_context();
            service.create_role(&context, request).await.unwrap();
        }

        let list_request = ListRolesRequest {
            path_prefix: Some("/test/".to_string()),
            pagination: None,
        };
        let (roles, _, _) = service.list_roles(list_request).await.unwrap();
        assert_eq!(roles.len(), 3);
    }

    #[tokio::test]
    async fn test_create_role_with_tags() {
        let service = setup_service();

        let tags = vec![Tag {
            key: "Environment".to_string(),
            value: "Production".to_string(),
        }];

        let request = CreateRoleRequest {
            role_name: "tagged-role".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            path: None,
            description: None,
            max_session_duration: None,
            permissions_boundary: None,
            tags: Some(tags.clone()),
        };

        let context = test_context();
        let role = service.create_role(&context, request).await.unwrap();
        assert_eq!(role.tags.len(), 1);
        assert_eq!(role.tags[0].key, "Environment");
    }
}
