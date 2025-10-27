//! AccessKey Operations

use super::{builder, model::*, requests::*};
use crate::error::Result;
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;

impl<S: Store> IamClient<S> {
    /// Creates a new access key for the specified IAM user
    pub async fn create_access_key(
        &mut self,
        request: CreateAccessKeyRequest,
    ) -> Result<AmiResponse<AccessKey>> {
        let account_id = self.account_id().await?;
        let provider = self.cloud_provider();

        let store = self.iam_store().await?;

        // Check if user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            });
        }

        // Check if user already has maximum number of access keys (provider-specific limit)
        let (existing_keys, _, _) = store.list_access_keys(&request.user_name, None).await?;
        let max_keys = provider.resource_limits().max_access_keys_per_user;
        if existing_keys.len() >= max_keys {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "User {} already has the maximum number of access keys ({})",
                    request.user_name, max_keys
                ),
            });
        }

        let access_key =
            builder::build_access_key(request.user_name, provider.as_ref(), &account_id);

        let created_key = store.create_access_key(access_key).await?;

        Ok(AmiResponse::success(created_key))
    }

    /// Deletes the specified access key for an IAM user
    pub async fn delete_access_key(
        &mut self,
        user_name: String,
        access_key_id: String,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Validate access key exists and belongs to user
        match store.get_access_key(&access_key_id).await? {
            Some(key) => {
                if key.user_name != user_name {
                    return Err(crate::error::AmiError::InvalidParameter {
                        message: "Access key does not belong to the specified user".to_string(),
                    });
                }
            }
            None => {
                return Err(crate::error::AmiError::ResourceNotFound {
                    resource: format!("AccessKey: {}", access_key_id),
                });
            }
        }

        store.delete_access_key(&access_key_id).await?;
        Ok(AmiResponse::success(()))
    }

    /// Updates the status of the specified access key
    pub async fn update_access_key(
        &mut self,
        request: UpdateAccessKeyRequest,
    ) -> Result<AmiResponse<AccessKey>> {
        let store = self.iam_store().await?;

        // Validate status value
        if request.status != "Active" && request.status != "Inactive" {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "Invalid status '{}'. Must be 'Active' or 'Inactive'",
                    request.status
                ),
            });
        }

        // Validate access key exists and belongs to user
        let key = match store.get_access_key(&request.access_key_id).await? {
            Some(key) => {
                if key.user_name != request.user_name {
                    return Err(crate::error::AmiError::InvalidParameter {
                        message: "Access key does not belong to the specified user".to_string(),
                    });
                }
                key
            }
            None => {
                return Err(crate::error::AmiError::ResourceNotFound {
                    resource: format!("AccessKey: {}", request.access_key_id),
                });
            }
        };

        let updated_key = builder::update_access_key_status(key, request.status);
        let result = store.update_access_key(updated_key).await?;

        Ok(AmiResponse::success(result))
    }

    /// Lists all access keys for the specified IAM user
    pub async fn list_access_keys(
        &mut self,
        request: ListAccessKeysRequest,
    ) -> Result<AmiResponse<ListAccessKeysResponse>> {
        let store = self.iam_store().await?;

        let (access_keys, is_truncated, marker) = store
            .list_access_keys(&request.user_name, request.pagination.as_ref())
            .await?;

        let response = ListAccessKeysResponse {
            access_keys,
            is_truncated,
            marker,
        };

        Ok(AmiResponse::success(response))
    }

    /// Retrieves information about when the specified access key was last used
    pub async fn get_access_key_last_used(
        &mut self,
        access_key_id: String,
    ) -> Result<AmiResponse<AccessKeyLastUsed>> {
        let store = self.iam_store().await?;

        // Validate access key exists
        if store.get_access_key(&access_key_id).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("AccessKey: {}", access_key_id),
            });
        }

        // In a real implementation, this would track actual usage
        // For now, return mock data
        let last_used = AccessKeyLastUsed {
            last_used_date: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            region: Some("us-east-1".to_string()),
            service_name: Some("iam".to_string()),
        };

        Ok(AmiResponse::success(last_used))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::user::CreateUserRequest;
    use crate::iam::IamClient;
    use crate::store::memory::InMemoryStore;

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
    async fn test_create_access_key() {
        let mut client = create_test_client();
        create_test_user(&mut client, "test-user").await;

        let request = CreateAccessKeyRequest {
            user_name: "test-user".to_string(),
        };

        let response = client.create_access_key(request).await.unwrap();
        assert!(response.success);

        let access_key = response.data.unwrap();
        assert_eq!(access_key.user_name, "test-user");
        assert!(access_key.access_key_id.starts_with("AKIA"));
        assert_eq!(access_key.status, "Active");
        assert!(access_key.secret_access_key.is_some());
    }

    #[tokio::test]
    async fn test_create_access_key_user_not_found() {
        let mut client = create_test_client();

        let request = CreateAccessKeyRequest {
            user_name: "nonexistent-user".to_string(),
        };

        let result = client.create_access_key(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_access_key_max_limit() {
        let mut client = create_test_client();
        create_test_user(&mut client, "test-user").await;

        // Create 2 access keys (AWS limit)
        for _ in 0..2 {
            let request = CreateAccessKeyRequest {
                user_name: "test-user".to_string(),
            };
            client.create_access_key(request).await.unwrap();
        }

        // Try to create a third one - should fail
        let request = CreateAccessKeyRequest {
            user_name: "test-user".to_string(),
        };
        let result = client.create_access_key(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_access_key() {
        let mut client = create_test_client();
        create_test_user(&mut client, "test-user").await;

        let create_request = CreateAccessKeyRequest {
            user_name: "test-user".to_string(),
        };
        let create_response = client.create_access_key(create_request).await.unwrap();
        let access_key_id = create_response.data.unwrap().access_key_id;

        let result = client
            .delete_access_key("test-user".to_string(), access_key_id)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_access_key_wrong_user() {
        let mut client = create_test_client();
        create_test_user(&mut client, "test-user").await;
        create_test_user(&mut client, "other-user").await;

        let create_request = CreateAccessKeyRequest {
            user_name: "test-user".to_string(),
        };
        let create_response = client.create_access_key(create_request).await.unwrap();
        let access_key_id = create_response.data.unwrap().access_key_id;

        // Try to delete with wrong user
        let result = client
            .delete_access_key("other-user".to_string(), access_key_id)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_access_key() {
        let mut client = create_test_client();
        create_test_user(&mut client, "test-user").await;

        let create_request = CreateAccessKeyRequest {
            user_name: "test-user".to_string(),
        };
        let create_response = client.create_access_key(create_request).await.unwrap();
        let access_key_id = create_response.data.unwrap().access_key_id;

        let update_request = UpdateAccessKeyRequest {
            user_name: "test-user".to_string(),
            access_key_id: access_key_id.clone(),
            status: "Inactive".to_string(),
        };
        let response = client.update_access_key(update_request).await.unwrap();
        assert_eq!(response.data.unwrap().status, "Inactive");
    }

    #[tokio::test]
    async fn test_update_access_key_invalid_status() {
        let mut client = create_test_client();
        create_test_user(&mut client, "test-user").await;

        let create_request = CreateAccessKeyRequest {
            user_name: "test-user".to_string(),
        };
        let create_response = client.create_access_key(create_request).await.unwrap();
        let access_key_id = create_response.data.unwrap().access_key_id;

        let update_request = UpdateAccessKeyRequest {
            user_name: "test-user".to_string(),
            access_key_id,
            status: "Invalid".to_string(),
        };
        let result = client.update_access_key(update_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_access_keys() {
        let mut client = create_test_client();
        create_test_user(&mut client, "test-user").await;

        // Create 2 access keys
        for _ in 0..2 {
            let request = CreateAccessKeyRequest {
                user_name: "test-user".to_string(),
            };
            client.create_access_key(request).await.unwrap();
        }

        let list_request = ListAccessKeysRequest {
            user_name: "test-user".to_string(),
            pagination: None,
        };
        let response = client.list_access_keys(list_request).await.unwrap();
        let list_response = response.data.unwrap();

        assert_eq!(list_response.access_keys.len(), 2);
        assert!(!list_response.is_truncated);
    }

    #[tokio::test]
    async fn test_get_access_key_last_used() {
        let mut client = create_test_client();
        create_test_user(&mut client, "test-user").await;

        let create_request = CreateAccessKeyRequest {
            user_name: "test-user".to_string(),
        };
        let create_response = client.create_access_key(create_request).await.unwrap();
        let access_key_id = create_response.data.unwrap().access_key_id;

        let response = client
            .get_access_key_last_used(access_key_id)
            .await
            .unwrap();
        let last_used = response.data.unwrap();

        assert!(last_used.last_used_date.is_some());
        assert_eq!(last_used.region, Some("us-east-1".to_string()));
        assert_eq!(last_used.service_name, Some("iam".to_string()));
    }
}
