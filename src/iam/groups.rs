use crate::error::Result;
use crate::iam::Group;
use crate::provider::ResourceType;
use crate::store::{IamStore, Store};
use crate::types::{AmiResponse, Tag};
use serde::{Deserialize, Serialize};

/// Parameters for creating a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub group_name: String,
    pub path: Option<String>,
    pub tags: Option<Vec<Tag>>,
}

/// Parameters for updating a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub group_name: String,
    pub new_group_name: Option<String>,
    pub new_path: Option<String>,
}

/// Parameters for listing groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGroupsRequest {
    pub path_prefix: Option<String>,
    pub pagination: Option<crate::types::PaginationParams>,
}

/// Response for listing groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGroupsResponse {
    pub groups: Vec<Group>,
    pub is_truncated: bool,
    pub marker: Option<String>,
}

impl<S: Store> crate::iam::IamClient<S> {
    /// Create a new group
    pub async fn create_group(
        &mut self,
        request: CreateGroupRequest,
    ) -> Result<AmiResponse<Group>> {
        let store = self.iam_store().await?;
        let account_id = store.account_id();
        let provider = store.cloud_provider();

        // Use provider for ID and ARN generation
        let group_id = provider.generate_resource_id(ResourceType::Group);
        let path = request.path.unwrap_or_else(|| "/".to_string());
        let arn = provider.generate_resource_identifier(
            ResourceType::Group,
            account_id,
            &path,
            &request.group_name,
        );

        // Generate WAMI ARN for cross-provider identification
        let wami_arn =
            provider.generate_wami_arn(ResourceType::Group, account_id, &path, &request.group_name);

        let group = Group {
            group_name: request.group_name.clone(),
            group_id: group_id.clone(),
            arn: arn.clone(),
            path,
            create_date: chrono::Utc::now(),
            tags: request.tags.unwrap_or_default(),
            wami_arn,
            providers: Vec::new(),
        };

        let created_group = store.create_group(group).await?;

        Ok(AmiResponse::success(created_group))
    }

    /// Update group properties
    pub async fn update_group(
        &mut self,
        request: UpdateGroupRequest,
    ) -> Result<AmiResponse<Group>> {
        let store = self.iam_store().await?;
        let provider = store.cloud_provider();
        let account_id = store.account_id();

        // Get the existing group
        let mut group = match store.get_group(&request.group_name).await? {
            Some(group) => group,
            None => {
                return Err(crate::error::AmiError::ResourceNotFound {
                    resource: format!("Group: {}", request.group_name),
                })
            }
        };

        // Update group properties
        if let Some(new_name) = request.new_group_name {
            group.group_name = new_name.clone();
            // Use provider for ARN generation
            group.arn = provider.generate_resource_identifier(
                ResourceType::Group,
                account_id,
                &group.path,
                &new_name,
            );
        }
        if let Some(new_path) = request.new_path {
            group.path = new_path;
        }

        let updated_group = store.update_group(group).await?;

        Ok(AmiResponse::success(updated_group))
    }

    /// Delete a group
    pub async fn delete_group(&mut self, group_name: String) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Check if group exists before deleting
        if store.get_group(&group_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            });
        }

        store.delete_group(&group_name).await?;
        Ok(AmiResponse::success(()))
    }

    /// Get group information
    pub async fn get_group(&mut self, group_name: String) -> Result<AmiResponse<Group>> {
        let store = self.iam_store().await?;

        match store.get_group(&group_name).await? {
            Some(group) => Ok(AmiResponse::success(group)),
            None => Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            }),
        }
    }

    /// List all groups
    pub async fn list_groups(
        &mut self,
        request: Option<ListGroupsRequest>,
    ) -> Result<AmiResponse<ListGroupsResponse>> {
        let store = self.iam_store().await?;

        let path_prefix = request.as_ref().and_then(|r| r.path_prefix.as_deref());
        let pagination = request.as_ref().and_then(|r| r.pagination.as_ref());

        let (groups, is_truncated, marker) = store.list_groups(path_prefix, pagination).await?;

        let response = ListGroupsResponse {
            groups,
            is_truncated,
            marker,
        };

        Ok(AmiResponse::success(response))
    }

    /// List groups for a user
    pub async fn list_groups_for_user(
        &mut self,
        user_name: String,
    ) -> Result<AmiResponse<Vec<Group>>> {
        let store = self.iam_store().await?;

        let groups = store.list_groups_for_user(&user_name).await?;
        Ok(AmiResponse::success(groups))
    }

    /// Add user to group
    pub async fn add_user_to_group(
        &mut self,
        group_name: String,
        user_name: String,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Check if group exists
        if store.get_group(&group_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            });
        }

        // Check if user exists
        if store.get_user(&user_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", user_name),
            });
        }

        store.add_user_to_group(&group_name, &user_name).await?;
        Ok(AmiResponse::success(()))
    }

    /// Remove user from group
    pub async fn remove_user_from_group(
        &mut self,
        group_name: String,
        user_name: String,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Check if group exists
        if store.get_group(&group_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            });
        }

        // Check if user exists
        if store.get_user(&user_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", user_name),
            });
        }

        store
            .remove_user_from_group(&group_name, &user_name)
            .await?;
        Ok(AmiResponse::success(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::{CreateUserRequest, IamClient};
    use crate::store::in_memory::InMemoryStore;

    fn create_test_client() -> IamClient<InMemoryStore> {
        let store = InMemoryStore::new();
        IamClient::new(store)
    }

    async fn create_test_user(client: &mut IamClient<InMemoryStore>, user_name: &str) {
        let request = CreateUserRequest {
            user_name: user_name.to_string(),
            path: Some("/".to_string()),
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(request).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_group() {
        let mut client = create_test_client();

        let request = CreateGroupRequest {
            group_name: "Developers".to_string(),
            path: Some("/engineering/".to_string()),
            tags: None,
        };

        let response = client.create_group(request).await.unwrap();
        assert!(response.success);

        let group = response.data.unwrap();
        assert_eq!(group.group_name, "Developers");
        assert_eq!(group.path, "/engineering/");
        assert!(group.group_id.starts_with("AGPA"));
    }

    #[tokio::test]
    async fn test_get_group() {
        let mut client = create_test_client();

        let create_request = CreateGroupRequest {
            group_name: "Admins".to_string(),
            path: Some("/".to_string()),
            tags: None,
        };
        client.create_group(create_request).await.unwrap();

        let response = client.get_group("Admins".to_string()).await.unwrap();
        let group = response.data.unwrap();
        assert_eq!(group.group_name, "Admins");
    }

    #[tokio::test]
    async fn test_get_nonexistent_group() {
        let mut client = create_test_client();

        let result = client.get_group("NonExistent".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_group() {
        let mut client = create_test_client();

        let create_request = CreateGroupRequest {
            group_name: "OldName".to_string(),
            path: Some("/old/".to_string()),
            tags: None,
        };
        client.create_group(create_request).await.unwrap();

        let update_request = UpdateGroupRequest {
            group_name: "OldName".to_string(),
            new_group_name: Some("NewName".to_string()),
            new_path: Some("/new/".to_string()),
        };

        let response = client.update_group(update_request).await.unwrap();
        let group = response.data.unwrap();
        assert_eq!(group.group_name, "NewName");
        assert_eq!(group.path, "/new/");
    }

    #[tokio::test]
    async fn test_delete_group() {
        let mut client = create_test_client();

        let create_request = CreateGroupRequest {
            group_name: "ToDelete".to_string(),
            path: Some("/".to_string()),
            tags: None,
        };
        client.create_group(create_request).await.unwrap();

        let result = client.delete_group("ToDelete".to_string()).await;
        assert!(result.is_ok());

        // Verify it's deleted
        let get_result = client.get_group("ToDelete".to_string()).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_list_groups() {
        let mut client = create_test_client();

        // Create multiple groups
        for i in 1..=3 {
            let request = CreateGroupRequest {
                group_name: format!("Group{}", i),
                path: Some("/".to_string()),
                tags: None,
            };
            client.create_group(request).await.unwrap();
        }

        let response = client.list_groups(None).await.unwrap();
        let list_response = response.data.unwrap();

        assert_eq!(list_response.groups.len(), 3);
        assert!(!list_response.is_truncated);
    }

    #[tokio::test]
    async fn test_add_user_to_group() {
        let mut client = create_test_client();

        // Create user and group
        create_test_user(&mut client, "testuser").await;
        let group_request = CreateGroupRequest {
            group_name: "TestGroup".to_string(),
            path: Some("/".to_string()),
            tags: None,
        };
        client.create_group(group_request).await.unwrap();

        // Add user to group
        let result = client
            .add_user_to_group("TestGroup".to_string(), "testuser".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_user_to_nonexistent_group() {
        let mut client = create_test_client();
        create_test_user(&mut client, "testuser").await;

        let result = client
            .add_user_to_group("NonExistent".to_string(), "testuser".to_string())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_user_from_group() {
        let mut client = create_test_client();

        // Create user and group
        create_test_user(&mut client, "testuser").await;
        let group_request = CreateGroupRequest {
            group_name: "TestGroup".to_string(),
            path: Some("/".to_string()),
            tags: None,
        };
        client.create_group(group_request).await.unwrap();

        // Add user to group
        client
            .add_user_to_group("TestGroup".to_string(), "testuser".to_string())
            .await
            .unwrap();

        // Remove user from group
        let result = client
            .remove_user_from_group("TestGroup".to_string(), "testuser".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_groups_for_user() {
        let mut client = create_test_client();

        // Create user and groups
        create_test_user(&mut client, "testuser").await;
        for i in 1..=2 {
            let request = CreateGroupRequest {
                group_name: format!("Group{}", i),
                path: Some("/".to_string()),
                tags: None,
            };
            client.create_group(request).await.unwrap();
            client
                .add_user_to_group(format!("Group{}", i), "testuser".to_string())
                .await
                .unwrap();
        }

        let response = client
            .list_groups_for_user("testuser".to_string())
            .await
            .unwrap();
        let groups = response.data.unwrap();
        assert_eq!(groups.len(), 2);
    }
}
