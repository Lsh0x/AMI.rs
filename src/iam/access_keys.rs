//! IAM Access Key Management
//!
//! This module provides functionality for managing AWS IAM access keys.
//! Access keys consist of an access key ID and secret access key, which are used to sign
//! programmatic requests to AWS.
//!
//! # Example
//!
//! ```rust,ignore
//! use wami::{MemoryIamClient, CreateAccessKeyRequest, CreateUserRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = wami::create_memory_store();
//! let mut iam_client = MemoryIamClient::new(store);
//!
//! // First, create a user
//! let user_request = CreateUserRequest {
//!     user_name: "alice".to_string(),
//!     path: Some("/".to_string()),
//!     permissions_boundary: None,
//!     tags: None,
//! };
//! iam_client.create_user(user_request).await?;
//!
//! // Create access keys for the user
//! let request = CreateAccessKeyRequest {
//!     user_name: "alice".to_string(),
//! };
//! let response = iam_client.create_access_key(request).await?;
//! let access_key = response.data.unwrap();
//!
//! println!("Access Key ID: {}", access_key.access_key_id);
//! println!("Secret: {}", access_key.secret_access_key.unwrap());
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::iam::AccessKey;
use crate::store::{IamStore, Store};
use crate::types::{AmiResponse, PaginationParams};
use serde::{Deserialize, Serialize};

/// Request parameters for creating a new access key
///
/// # Example
///
/// ```rust
/// use wami::CreateAccessKeyRequest;
///
/// let request = CreateAccessKeyRequest {
///     user_name: "my-user".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccessKeyRequest {
    /// The name of the IAM user to create the access key for
    pub user_name: String,
}

/// Request parameters for updating an access key's status
///
/// # Example
///
/// ```rust
/// use wami::UpdateAccessKeyRequest;
///
/// let request = UpdateAccessKeyRequest {
///     user_name: "my-user".to_string(),
///     access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
///     status: "Inactive".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccessKeyRequest {
    /// The name of the user whose access key should be updated
    pub user_name: String,
    /// The access key ID to update
    pub access_key_id: String,
    /// The new status: "Active" or "Inactive"
    pub status: String,
}

/// Request parameters for listing access keys
///
/// # Example
///
/// ```rust
/// use wami::{ListAccessKeysRequest, PaginationParams};
///
/// let request = ListAccessKeysRequest {
///     user_name: "my-user".to_string(),
///     pagination: Some(PaginationParams {
///         max_items: Some(10),
///         marker: None,
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccessKeysRequest {
    /// The name of the user whose access keys to list
    pub user_name: String,
    /// Optional pagination parameters
    pub pagination: Option<PaginationParams>,
}

/// Response containing a list of access keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccessKeysResponse {
    /// The list of access keys
    pub access_keys: Vec<AccessKey>,
    /// Whether the results are truncated
    pub is_truncated: bool,
    /// Marker for pagination
    pub marker: Option<String>,
}

/// Information about when an access key was last used
///
/// # Example
///
/// ```rust
/// use wami::AccessKeyLastUsed;
/// use chrono::Utc;
///
/// let last_used = AccessKeyLastUsed {
///     last_used_date: Some(Utc::now()),
///     region: Some("us-east-1".to_string()),
///     service_name: Some("s3".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKeyLastUsed {
    /// The date and time when the access key was last used
    pub last_used_date: Option<chrono::DateTime<chrono::Utc>>,
    /// The AWS region where the access key was last used
    pub region: Option<String>,
    /// The AWS service that was accessed
    pub service_name: Option<String>,
}

impl<S: Store> crate::iam::IamClient<S> {
    /// Creates a new access key for the specified IAM user
    ///
    /// Access keys are long-term credentials for an IAM user or AWS account root user.
    /// You can use access keys to sign programmatic requests to the AWS CLI or AWS API.
    ///
    /// # Arguments
    ///
    /// * `request` - The request containing the user name
    ///
    /// # Returns
    ///
    /// Returns the newly created access key, including the secret access key (only returned on creation).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The user does not exist
    /// * The user already has the maximum number of access keys (2)
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateAccessKeyRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Create a user first
    /// let user_request = CreateUserRequest {
    ///     user_name: "developer".to_string(),
    ///     path: Some("/engineering/".to_string()),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_user(user_request).await?;
    ///
    /// // Create access key
    /// let request = CreateAccessKeyRequest {
    ///     user_name: "developer".to_string(),
    /// };
    /// let response = iam_client.create_access_key(request).await?;
    /// let access_key = response.data.unwrap();
    ///
    /// // Store these credentials securely - the secret is only shown once!
    /// println!("Access Key ID: {}", access_key.access_key_id);
    /// println!("Secret Access Key: {}", access_key.secret_access_key.unwrap());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_access_key(
        &mut self,
        request: CreateAccessKeyRequest,
    ) -> Result<AmiResponse<AccessKey>> {
        let store = self.iam_store().await?;

        // Check if user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            });
        }

        // Check if user already has 2 access keys (AWS limit)
        let (existing_keys, _, _) = store.list_access_keys(&request.user_name, None).await?;
        if existing_keys.len() >= 2 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "User {} already has the maximum number of access keys (2)",
                    request.user_name
                ),
            });
        }

        // Generate access key ID (AKIA + 16 random chars)
        let access_key_id = format!(
            "AKIA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(16)
                .collect::<String>()
        );

        // Generate secret access key (40 random chars)
        let secret_access_key = uuid::Uuid::new_v4().to_string().replace('-', "")
            + &uuid::Uuid::new_v4().to_string().replace('-', "")[..8];

        let access_key = AccessKey {
            user_name: request.user_name.clone(),
            access_key_id: access_key_id.clone(),
            status: "Active".to_string(),
            create_date: chrono::Utc::now(),
            secret_access_key: Some(secret_access_key),
        };

        let created_key = store.create_access_key(access_key).await?;

        Ok(AmiResponse::success(created_key))
    }

    /// Deletes the specified access key for an IAM user
    ///
    /// # Arguments
    ///
    /// * `user_name` - The name of the user whose access key should be deleted
    /// * `access_key_id` - The ID of the access key to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The access key does not exist
    /// * The access key does not belong to the specified user
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateAccessKeyRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Setup: create user and access key
    /// let user_request = CreateUserRequest {
    ///     user_name: "old-user".to_string(),
    ///     path: Some("/".to_string()),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_user(user_request).await?;
    ///
    /// let key_request = CreateAccessKeyRequest {
    ///     user_name: "old-user".to_string(),
    /// };
    /// let key_response = iam_client.create_access_key(key_request).await?;
    /// let access_key_id = key_response.data.unwrap().access_key_id;
    ///
    /// // Delete the access key
    /// iam_client.delete_access_key("old-user".to_string(), access_key_id).await?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Changes the status of an access key from Active to Inactive, or vice versa.
    /// Inactive access keys cannot be used for authentication.
    ///
    /// # Arguments
    ///
    /// * `request` - The update request containing user name, access key ID, and new status
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The access key does not exist
    /// * The access key does not belong to the specified user
    /// * The status value is invalid (must be "Active" or "Inactive")
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateAccessKeyRequest, UpdateAccessKeyRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Setup
    /// let user_request = CreateUserRequest {
    ///     user_name: "api-user".to_string(),
    ///     path: Some("/".to_string()),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_user(user_request).await?;
    ///
    /// let key_request = CreateAccessKeyRequest {
    ///     user_name: "api-user".to_string(),
    /// };
    /// let key_response = iam_client.create_access_key(key_request).await?;
    /// let access_key_id = key_response.data.unwrap().access_key_id;
    ///
    /// // Deactivate the access key (e.g., for key rotation)
    /// let update_request = UpdateAccessKeyRequest {
    ///     user_name: "api-user".to_string(),
    ///     access_key_id: access_key_id.clone(),
    ///     status: "Inactive".to_string(),
    /// };
    /// iam_client.update_access_key(update_request).await?;
    /// # Ok(())
    /// # }
    /// ```
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
        let mut key = match store.get_access_key(&request.access_key_id).await? {
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

        // Update the status
        key.status = request.status.clone();
        let updated_key = store.update_access_key(key).await?;

        Ok(AmiResponse::success(updated_key))
    }

    /// Lists all access keys for the specified IAM user
    ///
    /// # Arguments
    ///
    /// * `request` - The list request containing the user name and optional pagination
    ///
    /// # Returns
    ///
    /// Returns a list of access keys (without the secret access key values).
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateAccessKeyRequest, ListAccessKeysRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Setup
    /// let user_request = CreateUserRequest {
    ///     user_name: "app-user".to_string(),
    ///     path: Some("/".to_string()),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_user(user_request).await?;
    ///
    /// // Create multiple access keys
    /// for _ in 0..2 {
    ///     let key_request = CreateAccessKeyRequest {
    ///         user_name: "app-user".to_string(),
    ///     };
    ///     iam_client.create_access_key(key_request).await?;
    /// }
    ///
    /// // List all access keys
    /// let list_request = ListAccessKeysRequest {
    ///     user_name: "app-user".to_string(),
    ///     pagination: None,
    /// };
    /// let response = iam_client.list_access_keys(list_request).await?;
    /// let keys = response.data.unwrap();
    ///
    /// println!("Found {} access keys", keys.access_keys.len());
    /// for key in keys.access_keys {
    ///     println!("- {} ({})", key.access_key_id, key.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// The returned information includes the date and time of last use, the AWS region,
    /// and the service that was accessed.
    ///
    /// # Arguments
    ///
    /// * `access_key_id` - The identifier of the access key
    ///
    /// # Errors
    ///
    /// Returns an error if the access key does not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateAccessKeyRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Setup
    /// let user_request = CreateUserRequest {
    ///     user_name: "audit-user".to_string(),
    ///     path: Some("/".to_string()),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_user(user_request).await?;
    ///
    /// let key_request = CreateAccessKeyRequest {
    ///     user_name: "audit-user".to_string(),
    /// };
    /// let key_response = iam_client.create_access_key(key_request).await?;
    /// let access_key_id = key_response.data.unwrap().access_key_id;
    ///
    /// // Check last usage
    /// let last_used_response = iam_client.get_access_key_last_used(access_key_id).await?;
    /// let last_used = last_used_response.data.unwrap();
    ///
    /// if let Some(date) = last_used.last_used_date {
    ///     println!("Last used: {}", date);
    ///     println!("Region: {}", last_used.region.unwrap_or_default());
    ///     println!("Service: {}", last_used.service_name.unwrap_or_default());
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
