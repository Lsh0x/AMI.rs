use crate::error::Result;
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::{AmiResponse, Tag};
use serde::{Deserialize, Serialize};

/// Request to tag an IAM resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagResourceRequest {
    /// The type of resource to tag ("user", "group", "role", "policy")
    pub resource_type: String,
    /// The identifier of the resource (user_name, group_name, role_name, or policy_arn)
    pub resource_id: String,
    /// The tags to add to the resource
    pub tags: Vec<Tag>,
}

/// Request to untag an IAM resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UntagResourceRequest {
    /// The type of resource to untag ("user", "group", "role", "policy")
    pub resource_type: String,
    /// The identifier of the resource (user_name, group_name, role_name, or policy_arn)
    pub resource_id: String,
    /// The tag keys to remove from the resource
    pub tag_keys: Vec<String>,
}

/// Request to list tags for an IAM resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourceTagsRequest {
    /// The type of resource ("user", "group", "role", "policy")
    pub resource_type: String,
    /// The identifier of the resource (user_name, group_name, role_name, or policy_arn)
    pub resource_id: String,
}

impl<S: Store> IamClient<S> {
    /// Tag an IAM group
    ///
    /// # Arguments
    ///
    /// * `group_name` - The name of the group to tag
    /// * `tags` - The tags to add
    ///
    /// # Returns
    ///
    /// Returns success if the tags were added
    ///
    /// # Errors
    ///
    /// * `ResourceNotFound` - If the group doesn't exist
    /// * `InvalidParameter` - If tags are invalid
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateGroupRequest, Tag};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create a group
    /// let group_request = CreateGroupRequest {
    ///     group_name: "Developers".to_string(),
    ///     path: None,
    ///     tags: None,
    /// };
    /// client.create_group(group_request).await?;
    ///
    /// // Tag the group
    /// let tags = vec![
    ///     Tag { key: "Department".to_string(), value: "Engineering".to_string() },
    /// ];
    /// let response = client.tag_group("Developers".to_string(), tags).await?;
    /// assert!(response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn tag_group(
        &mut self,
        group_name: String,
        tags: Vec<Tag>,
    ) -> Result<AmiResponse<()>> {
        Self::validate_tags(&tags)?;

        let store = self.iam_store().await?;

        if store.get_group(&group_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            });
        }

        store.tag_group(&group_name, tags).await?;

        Ok(AmiResponse::success(()))
    }

    /// Remove tags from an IAM group
    pub async fn untag_group(
        &mut self,
        group_name: String,
        tag_keys: Vec<String>,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        if store.get_group(&group_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            });
        }

        store.untag_group(&group_name, tag_keys).await?;

        Ok(AmiResponse::success(()))
    }

    /// List tags for an IAM group
    pub async fn list_group_tags(&mut self, group_name: String) -> Result<AmiResponse<Vec<Tag>>> {
        let store = self.iam_store().await?;

        if store.get_group(&group_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Group: {}", group_name),
            });
        }

        let tags = store.list_group_tags(&group_name).await?;

        Ok(AmiResponse::success(tags))
    }

    /// Tag an IAM role
    pub async fn tag_role(&mut self, role_name: String, tags: Vec<Tag>) -> Result<AmiResponse<()>> {
        Self::validate_tags(&tags)?;

        let store = self.iam_store().await?;

        if store.get_role(&role_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Role: {}", role_name),
            });
        }

        store.tag_role(&role_name, tags).await?;

        Ok(AmiResponse::success(()))
    }

    /// Remove tags from an IAM role
    pub async fn untag_role(
        &mut self,
        role_name: String,
        tag_keys: Vec<String>,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        if store.get_role(&role_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Role: {}", role_name),
            });
        }

        store.untag_role(&role_name, tag_keys).await?;

        Ok(AmiResponse::success(()))
    }

    /// List tags for an IAM role
    pub async fn list_role_tags(&mut self, role_name: String) -> Result<AmiResponse<Vec<Tag>>> {
        let store = self.iam_store().await?;

        if store.get_role(&role_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Role: {}", role_name),
            });
        }

        let tags = store.list_role_tags(&role_name).await?;

        Ok(AmiResponse::success(tags))
    }

    /// Tag an IAM policy
    pub async fn tag_policy(
        &mut self,
        policy_arn: String,
        tags: Vec<Tag>,
    ) -> Result<AmiResponse<()>> {
        Self::validate_tags(&tags)?;

        let store = self.iam_store().await?;

        if store.get_policy(&policy_arn).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Policy: {}", policy_arn),
            });
        }

        store.tag_policy(&policy_arn, tags).await?;

        Ok(AmiResponse::success(()))
    }

    /// Remove tags from an IAM policy
    pub async fn untag_policy(
        &mut self,
        policy_arn: String,
        tag_keys: Vec<String>,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        if store.get_policy(&policy_arn).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Policy: {}", policy_arn),
            });
        }

        store.untag_policy(&policy_arn, tag_keys).await?;

        Ok(AmiResponse::success(()))
    }

    /// List tags for an IAM policy
    pub async fn list_policy_tags(&mut self, policy_arn: String) -> Result<AmiResponse<Vec<Tag>>> {
        let store = self.iam_store().await?;

        if store.get_policy(&policy_arn).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Policy: {}", policy_arn),
            });
        }

        let tags = store.list_policy_tags(&policy_arn).await?;

        Ok(AmiResponse::success(tags))
    }

    /// Validate tags
    ///
    /// AWS tag requirements:
    /// - Maximum 50 tags per resource
    /// - Tag keys are case sensitive
    /// - Tag keys must be unique
    /// - Maximum key length: 128 characters
    /// - Maximum value length: 256 characters
    #[allow(clippy::result_large_err)]
    fn validate_tags(tags: &[Tag]) -> Result<()> {
        if tags.len() > 50 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Cannot add more than 50 tags to a resource".to_string(),
            });
        }

        // Check for duplicate keys
        let mut keys = std::collections::HashSet::new();
        for tag in tags {
            if !keys.insert(&tag.key) {
                return Err(crate::error::AmiError::InvalidParameter {
                    message: format!("Duplicate tag key: {}", tag.key),
                });
            }

            if tag.key.is_empty() || tag.key.len() > 128 {
                return Err(crate::error::AmiError::InvalidParameter {
                    message: "Tag key must be between 1 and 128 characters".to_string(),
                });
            }

            if tag.value.len() > 256 {
                return Err(crate::error::AmiError::InvalidParameter {
                    message: "Tag value must not exceed 256 characters".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::groups::CreateGroupRequest;
    use crate::iam::policies::CreatePolicyRequest;
    use crate::iam::roles::CreateRoleRequest;
    use crate::store::in_memory::InMemoryStore;

    #[tokio::test]
    async fn test_tag_group() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create group
        let group_request = CreateGroupRequest {
            group_name: "Developers".to_string(),
            path: None,
            tags: None,
        };
        client.create_group(group_request).await.unwrap();

        // Tag group
        let tags = vec![Tag {
            key: "Department".to_string(),
            value: "Engineering".to_string(),
        }];

        let response = client
            .tag_group("Developers".to_string(), tags)
            .await
            .unwrap();
        assert!(response.success);

        // List tags
        let list_response = client
            .list_group_tags("Developers".to_string())
            .await
            .unwrap();
        let tags = list_response.data.unwrap();
        assert_eq!(tags.len(), 1);
    }

    #[tokio::test]
    async fn test_tag_role() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create role
        let trust_policy = r#"{"Version": "2012-10-17", "Statement": []}"#;
        let role_request = CreateRoleRequest {
            role_name: "TestRole".to_string(),
            assume_role_policy_document: trust_policy.to_string(),
            path: None,
            description: None,
            max_session_duration: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_role(role_request).await.unwrap();

        // Tag role
        let tags = vec![Tag {
            key: "Environment".to_string(),
            value: "Production".to_string(),
        }];

        let response = client.tag_role("TestRole".to_string(), tags).await.unwrap();
        assert!(response.success);

        // List tags
        let list_response = client.list_role_tags("TestRole".to_string()).await.unwrap();
        let tags = list_response.data.unwrap();
        assert_eq!(tags.len(), 1);
    }

    #[tokio::test]
    async fn test_tag_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create policy
        let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
        let policy_request = CreatePolicyRequest {
            policy_name: "TestPolicy".to_string(),
            policy_document: policy_document.to_string(),
            path: None,
            description: None,
            tags: None,
        };
        let create_response = client.create_policy(policy_request).await.unwrap();
        let policy_arn = create_response.data.unwrap().arn;

        // Tag policy
        let tags = vec![Tag {
            key: "ManagedBy".to_string(),
            value: "Terraform".to_string(),
        }];

        let response = client.tag_policy(policy_arn.clone(), tags).await.unwrap();
        assert!(response.success);

        // List tags
        let list_response = client.list_policy_tags(policy_arn).await.unwrap();
        let tags = list_response.data.unwrap();
        assert_eq!(tags.len(), 1);
    }

    #[tokio::test]
    async fn test_validate_tags_too_many() {
        let tags: Vec<Tag> = (0..51)
            .map(|i| Tag {
                key: format!("Key{}", i),
                value: format!("Value{}", i),
            })
            .collect();

        let result = IamClient::<InMemoryStore>::validate_tags(&tags);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_tags_duplicate_keys() {
        let tags = vec![
            Tag {
                key: "Environment".to_string(),
                value: "Prod".to_string(),
            },
            Tag {
                key: "Environment".to_string(),
                value: "Dev".to_string(),
            },
        ];

        let result = IamClient::<InMemoryStore>::validate_tags(&tags);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_tags_key_too_long() {
        let tags = vec![Tag {
            key: "a".repeat(129),
            value: "value".to_string(),
        }];

        let result = IamClient::<InMemoryStore>::validate_tags(&tags);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_tags_value_too_long() {
        let tags = vec![Tag {
            key: "key".to_string(),
            value: "a".repeat(257),
        }];

        let result = IamClient::<InMemoryStore>::validate_tags(&tags);
        assert!(result.is_err());
    }
}
