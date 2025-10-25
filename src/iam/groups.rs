use crate::iam::Group;
use crate::types::Tag;
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

// TODO: These implementations need to be refactored to use the Store trait properly
// For now, these are commented out as they conflict with the generic Store implementation

/*
impl<S: Store> IamClient<S> {
    /// Create a new group
    pub async fn create_group(
        &mut self,
        request: CreateGroupRequest,
    ) -> Result<AmiResponse<Group>> {
        let store = self.iam_store().await?;
        let account_id = store.account_id();

        let group_id = format!(
            "AGPA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );
        let arn = format!("arn:aws:iam::{}:group/{}", account_id, request.group_name);

        let group = Group {
            group_name: request.group_name.clone(),
            group_id: group_id.clone(),
            arn: arn.clone(),
            path: request.path.unwrap_or_else(|| "/".to_string()),
            create_date: chrono::Utc::now(),
            tags: request.tags.unwrap_or_default(),
        };

        let created_group = store.create_group(group).await?;

        Ok(AmiResponse::success(created_group))
    }

    /// Update group properties
    pub async fn update_group(
        &mut self,
        request: UpdateGroupRequest,
    ) -> Result<AmiResponse<Group>> {
        if let Some(mut group) = self.groups.remove(&request.group_name) {
            // Update group properties
            if let Some(new_name) = request.new_group_name {
                group.group_name = new_name.clone();
                group.arn = format!("arn:aws:iam::123456789012:group/{}", new_name);
            }
            if let Some(new_path) = request.new_path {
                group.path = new_path;
            }

            let updated_group = group.clone();
            self.groups
                .insert(updated_group.group_name.clone(), updated_group.clone());

            Ok(AmiResponse::success(updated_group))
        } else {
            Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })
        }
    }

    /// Delete a group
    pub async fn delete_group(&mut self, group_name: String) -> Result<AmiResponse<()>> {
        if self.groups.remove(&group_name).is_some() {
            Ok(AmiResponse::success(()))
        } else {
            Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            })
        }
    }

    /// Get group information
    pub async fn get_group(&self, group_name: String) -> Result<AmiResponse<Group>> {
        match self.groups.get(&group_name) {
            Some(group) => Ok(AmiResponse::success(group.clone())),
            None => Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            }),
        }
    }

    /// List all groups
    pub async fn list_groups(
        &self,
        request: Option<ListGroupsRequest>,
    ) -> Result<AmiResponse<ListGroupsResponse>> {
        let mut groups: Vec<Group> = self.groups.values().cloned().collect();

        // Apply path prefix filter if specified
        if let Some(req) = &request {
            if let Some(path_prefix) = &req.path_prefix {
                groups.retain(|group| group.path.starts_with(path_prefix));
            }
        }

        // Sort by group name
        groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));

        // Apply pagination
        let mut is_truncated = false;
        let mut marker = None;

        if let Some(req) = &request {
            if let Some(pagination) = &req.pagination {
                if let Some(max_items) = pagination.max_items {
                    if groups.len() > max_items as usize {
                        groups.truncate(max_items as usize);
                        is_truncated = true;
                        marker = Some(groups.last().unwrap().group_name.clone());
                    }
                }
            }
        }

        let response = ListGroupsResponse {
            groups,
            is_truncated,
            marker,
        };

        Ok(AmiResponse::success(response))
    }

    /// List groups for a user
    pub async fn list_groups_for_user(&self, user_name: String) -> Result<AmiResponse<Vec<Group>>> {
        // In a real implementation, this would maintain user-group relationships
        // For now, return empty list
        Ok(AmiResponse::success(vec![]))
    }

    /// Add user to group
    pub async fn add_user_to_group(
        &mut self,
        group_name: String,
        user_name: String,
    ) -> Result<AmiResponse<()>> {
        // Check if group exists
        if !self.groups.contains_key(&group_name) {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            });
        }

        // Check if user exists
        if !self.users.contains_key(&user_name) {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", user_name),
            });
        }

        // In a real implementation, this would maintain user-group relationships
        Ok(AmiResponse::success(()))
    }

    /// Remove user from group
    pub async fn remove_user_from_group(
        &mut self,
        group_name: String,
        user_name: String,
    ) -> Result<AmiResponse<()>> {
        // Check if group exists
        if !self.groups.contains_key(&group_name) {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            });
        }

        // Check if user exists
        if !self.users.contains_key(&user_name) {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", user_name),
            });
        }

        // In a real implementation, this would maintain user-group relationships
        Ok(AmiResponse::success(()))
    }
}
*/
