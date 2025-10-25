//! IAM Access Key Management
//!
//! This module provides functionality for managing AWS IAM access keys.
//! Access keys consist of an access key ID and secret access key, which are used to sign
//! programmatic requests to AWS.
//!
//! # Example
//!
//! ```rust,ignore
//! use rustyiam::{MemoryIamClient, CreateAccessKeyRequest, CreateUserRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = rustyiam::create_memory_store();
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

use crate::iam::AccessKey;
use crate::types::PaginationParams;
use serde::{Deserialize, Serialize};

/// Request parameters for creating a new access key
///
/// # Example
///
/// ```rust
/// use rustyiam::CreateAccessKeyRequest;
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
/// use rustyiam::UpdateAccessKeyRequest;
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
/// use rustyiam::{ListAccessKeysRequest, PaginationParams};
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
/// use rustyiam::AccessKeyLastUsed;
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

// TODO: These implementations need to be refactored to use the Store trait properly
// For now, these are commented out as they conflict with the generic Store implementation

/*
impl<S: Store> IamClient<S> {
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
    /// use rustyiam::{MemoryIamClient, CreateAccessKeyRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
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
        // Check if user exists
        if !self.users.contains_key(&request.user_name) {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            });
        }

        let access_key_id = format!(
            "AKIA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );
        let secret_access_key = format!("{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

        let access_key = AccessKey {
            user_name: request.user_name.clone(),
            access_key_id: access_key_id.clone(),
            status: "Active".to_string(),
            create_date: chrono::Utc::now(),
            secret_access_key: Some(secret_access_key),
        };

        self.access_keys.insert(access_key_id, access_key.clone());

        Ok(AmiResponse::success(access_key))
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
    /// use rustyiam::{MemoryIamClient, CreateAccessKeyRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
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
        if let Some(key) = self.access_keys.get(&access_key_id) {
            if key.user_name == user_name {
                self.access_keys.remove(&access_key_id);
                Ok(AmiResponse::success(()))
            } else {
                Err(crate::error::AmiError::InvalidParameter {
                    message: "Access key does not belong to the specified user".to_string(),
                })
            }
        } else {
            Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("AccessKey: {}", access_key_id),
            })
        }
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
    /// use rustyiam::{MemoryIamClient, CreateAccessKeyRequest, UpdateAccessKeyRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
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
        if let Some(key) = self.access_keys.get_mut(&request.access_key_id) {
            if key.user_name == request.user_name {
                key.status = request.status.clone();
                Ok(AmiResponse::success(key.clone()))
            } else {
                Err(crate::error::AmiError::InvalidParameter {
                    message: "Access key does not belong to the specified user".to_string(),
                })
            }
        } else {
            Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("AccessKey: {}", request.access_key_id),
            })
        }
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
    /// use rustyiam::{MemoryIamClient, CreateAccessKeyRequest, ListAccessKeysRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
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
        &self,
        request: ListAccessKeysRequest,
    ) -> Result<AmiResponse<ListAccessKeysResponse>> {
        let mut access_keys: Vec<AccessKey> = self
            .access_keys
            .values()
            .filter(|key| key.user_name == request.user_name)
            .cloned()
            .collect();

        // Sort by access key id
        access_keys.sort_by(|a, b| a.access_key_id.cmp(&b.access_key_id));

        // Apply pagination
        let mut is_truncated = false;
        let mut marker = None;

        if let Some(pagination) = &request.pagination {
            if let Some(max_items) = pagination.max_items {
                if access_keys.len() > max_items as usize {
                    access_keys.truncate(max_items as usize);
                    is_truncated = true;
                    marker = Some(access_keys.last().unwrap().access_key_id.clone());
                }
            }
        }

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
    /// use rustyiam::{MemoryIamClient, CreateAccessKeyRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
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
        &self,
        access_key_id: String,
    ) -> Result<AmiResponse<AccessKeyLastUsed>> {
        if let Some(_key) = self.access_keys.get(&access_key_id) {
            // In a real implementation, this would track actual usage
            let last_used = AccessKeyLastUsed {
                last_used_date: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
                region: Some("us-east-1".to_string()),
                service_name: Some("iam".to_string()),
            };
            Ok(AmiResponse::success(last_used))
        } else {
            Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("AccessKey: {}", access_key_id),
            })
        }
    }
}
*/
