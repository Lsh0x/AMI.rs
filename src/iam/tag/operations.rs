//! Tag Operations

use crate::error::Result;
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::{AmiResponse, Tag};

impl<S: Store> IamClient<S> {
    /// Tag an IAM group
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
    #[allow(clippy::result_large_err)]
    fn validate_tags(tags: &[Tag]) -> Result<()> {
        if tags.len() > 50 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Cannot add more than 50 tags to a resource".to_string(),
            });
        }

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
    use crate::iam::group::CreateGroupRequest;
    use crate::store::memory::InMemoryStore;

    #[tokio::test]
    async fn test_tag_group() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        client
            .create_group(CreateGroupRequest {
                group_name: "Developers".to_string(),
                path: None,
                tags: None,
            })
            .await
            .unwrap();

        let tags = vec![Tag {
            key: "Department".to_string(),
            value: "Engineering".to_string(),
        }];

        let response = client
            .tag_group("Developers".to_string(), tags)
            .await
            .unwrap();
        assert!(response.success);
    }
}
