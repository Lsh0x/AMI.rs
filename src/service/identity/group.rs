//! Group Service
//!
//! Orchestrates group management operations including membership management.

use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::GroupStore;
use crate::wami::identity::group::{
    builder as group_builder, CreateGroupRequest, Group, ListGroupsRequest, UpdateGroupRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM groups
///
/// Provides high-level operations for group management and membership.
pub struct GroupService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: GroupStore> GroupService<S> {
    /// Create a new GroupService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Create a new group
    pub async fn create_group(
        &self,
        context: &WamiContext,
        request: CreateGroupRequest,
    ) -> Result<Group> {
        // Use wami builder to create group
        let group = group_builder::build_group(request.group_name, request.path, context)?;

        // Store it
        self.store.write().unwrap().create_group(group).await
    }

    /// Get a group by name
    pub async fn get_group(&self, group_name: &str) -> Result<Option<Group>> {
        self.store.read().unwrap().get_group(group_name).await
    }

    /// Update a group
    pub async fn update_group(&self, request: UpdateGroupRequest) -> Result<Group> {
        // Get existing group
        let mut group = self
            .store
            .read()
            .unwrap()
            .get_group(&request.group_name)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })?;

        // Apply updates using builder functions
        if let Some(new_group_name) = request.new_group_name {
            group = group_builder::update_group_name(group, new_group_name);
        }

        if let Some(new_path) = request.new_path {
            group = group_builder::update_group_path(group, new_path);
        }

        // Store updated group
        self.store.write().unwrap().update_group(group).await
    }

    /// Delete a group
    pub async fn delete_group(&self, group_name: &str) -> Result<()> {
        self.store.write().unwrap().delete_group(group_name).await
    }

    /// List groups with optional filtering
    pub async fn list_groups(
        &self,
        request: ListGroupsRequest,
    ) -> Result<(Vec<Group>, bool, Option<String>)> {
        self.store
            .read()
            .unwrap()
            .list_groups(request.path_prefix.as_deref(), request.pagination.as_ref())
            .await
    }

    /// Add a user to a group
    pub async fn add_user_to_group(&self, group_name: &str, user_name: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .add_user_to_group(group_name, user_name)
            .await
    }

    /// Remove a user from a group
    pub async fn remove_user_from_group(&self, group_name: &str, user_name: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .remove_user_from_group(group_name, user_name)
            .await
    }

    /// List all groups for a user
    pub async fn list_groups_for_user(&self, user_name: &str) -> Result<Vec<Group>> {
        self.store
            .read()
            .unwrap()
            .list_groups_for_user(user_name)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::store::memory::InMemoryWamiStore;
    use crate::store::traits::UserStore;
    use crate::wami::identity::user::builder as user_builder;

    fn setup_service() -> GroupService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        GroupService::new(store)
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
    async fn test_create_and_get_group() {
        let service = setup_service();
        let context = test_context();

        let request = CreateGroupRequest {
            group_name: "admins".to_string(),
            path: Some("/it/".to_string()),
            tags: None,
        };

        let group = service.create_group(&context, request).await.unwrap();
        assert_eq!(group.group_name, "admins");
        assert_eq!(group.path, "/it/");

        let retrieved = service.get_group("admins").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().group_name, "admins");
    }

    #[tokio::test]
    async fn test_update_group() {
        let service = setup_service();
        let context = test_context();

        // Create group
        let create_request = CreateGroupRequest {
            group_name: "developers".to_string(),
            path: Some("/".to_string()),
            tags: None,
        };
        service
            .create_group(&context, create_request)
            .await
            .unwrap();

        // Update group
        let update_request = UpdateGroupRequest {
            group_name: "developers".to_string(),
            new_group_name: Some("engineers".to_string()),
            new_path: Some("/tech/".to_string()),
        };
        let updated = service.update_group(update_request).await.unwrap();
        assert_eq!(updated.group_name, "engineers");
        assert_eq!(updated.path, "/tech/");
    }

    #[tokio::test]
    async fn test_delete_group() {
        let service = setup_service();

        let request = CreateGroupRequest {
            group_name: "temp_group".to_string(),
            path: None,
            tags: None,
        };
        let context = test_context();
        service.create_group(&context, request).await.unwrap();

        service.delete_group("temp_group").await.unwrap();

        let retrieved = service.get_group("temp_group").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_groups() {
        let service = setup_service();

        // Create multiple groups
        for name in ["group1", "group2", "group3"] {
            let request = CreateGroupRequest {
                group_name: name.to_string(),
                path: Some("/test/".to_string()),
                tags: None,
            };
            let context = test_context();
            service.create_group(&context, request).await.unwrap();
        }

        let list_request = ListGroupsRequest {
            path_prefix: Some("/test/".to_string()),
            pagination: None,
        };
        let (groups, _, _) = service.list_groups(list_request).await.unwrap();
        assert_eq!(groups.len(), 3);
    }

    #[tokio::test]
    async fn test_group_membership() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        let service = GroupService::new(store.clone());
        let context = test_context();

        // Create a user first
        let user =
            user_builder::build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        store.write().unwrap().create_user(user).await.unwrap();

        // Create a group
        let request = CreateGroupRequest {
            group_name: "admins".to_string(),
            path: None,
            tags: None,
        };
        service.create_group(&context, request).await.unwrap();

        // Add user to group
        service.add_user_to_group("admins", "alice").await.unwrap();

        // List groups for user
        let groups = service.list_groups_for_user("alice").await.unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].group_name, "admins");

        // Remove user from group
        service
            .remove_user_from_group("admins", "alice")
            .await
            .unwrap();

        let groups_after = service.list_groups_for_user("alice").await.unwrap();
        assert_eq!(groups_after.len(), 0);
    }
}
