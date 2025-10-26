//! IAM Service-Specific Credentials Management
//!
//! This module provides functionality for managing service-specific credentials
//! used for services like AWS CodeCommit and Amazon Keyspaces.

use crate::error::{AmiError, Result};
use crate::iam::IamClient;
use crate::provider::ResourceType;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpecificCredential {
    /// The name of the IAM user associated with the credential
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier for the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,

    /// The generated username for the service
    #[serde(rename = "ServiceUserName")]
    pub service_user_name: String,

    /// The generated password for the service (only returned on creation)
    #[serde(rename = "ServicePassword", skip_serializing_if = "Option::is_none")]
    pub service_password: Option<String>,

    /// The name of the service
    #[serde(rename = "ServiceName")]
    pub service_name: String,

    /// The date and time when the credential was created
    #[serde(rename = "CreateDate")]
    pub create_date: DateTime<Utc>,

    /// The status of the credential (Active or Inactive)
    #[serde(rename = "Status")]
    pub status: String,

    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,

    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Metadata about a service-specific credential (without password)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpecificCredentialMetadata {
    /// The name of the IAM user associated with the credential
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier for the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,

    /// The generated username for the service
    #[serde(rename = "ServiceUserName")]
    pub service_user_name: String,

    /// The name of the service
    #[serde(rename = "ServiceName")]
    pub service_name: String,

    /// The date and time when the credential was created
    #[serde(rename = "CreateDate")]
    pub create_date: DateTime<Utc>,

    /// The status of the credential (Active or Inactive)
    #[serde(rename = "Status")]
    pub status: String,
}

/// Request to create a service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceSpecificCredentialRequest {
    /// The name of the IAM user to associate with the credential
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The name of the AWS service (e.g., "codecommit.amazonaws.com")
    #[serde(rename = "ServiceName")]
    pub service_name: String,
}

/// Response from creating a service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceSpecificCredentialResponse {
    /// The created credential with password
    #[serde(rename = "ServiceSpecificCredential")]
    pub service_specific_credential: ServiceSpecificCredential,
}

/// Request to delete a service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteServiceSpecificCredentialRequest {
    /// The name of the IAM user
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier of the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,
}

/// Request to list service-specific credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServiceSpecificCredentialsRequest {
    /// The name of the IAM user (optional, lists all if not provided)
    #[serde(rename = "UserName", skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    /// Filter by service name (optional)
    #[serde(rename = "ServiceName", skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
}

/// Response from listing service-specific credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServiceSpecificCredentialsResponse {
    /// List of credential metadata
    #[serde(rename = "ServiceSpecificCredentials")]
    pub service_specific_credentials: Vec<ServiceSpecificCredentialMetadata>,
}

/// Request to reset a service-specific credential password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetServiceSpecificCredentialRequest {
    /// The name of the IAM user
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier of the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,
}

/// Response from resetting a service-specific credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetServiceSpecificCredentialResponse {
    /// The credential with new password
    #[serde(rename = "ServiceSpecificCredential")]
    pub service_specific_credential: ServiceSpecificCredential,
}

/// Request to update a service-specific credential status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceSpecificCredentialRequest {
    /// The name of the IAM user
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The unique identifier of the credential
    #[serde(rename = "ServiceSpecificCredentialId")]
    pub service_specific_credential_id: String,

    /// The new status (Active or Inactive)
    #[serde(rename = "Status")]
    pub status: String,
}

impl<S: Store> IamClient<S> {
    /// Create a service-specific credential
    ///
    /// Generates a username and password for a specific AWS service.
    ///
    /// # Arguments
    ///
    /// * `request` - The create service-specific credential request
    ///
    /// # Returns
    ///
    /// Returns the created credential with password (only returned on creation)
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateServiceSpecificCredentialRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create a user first
    /// client.create_user(CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// }).await?;
    ///
    /// let request = CreateServiceSpecificCredentialRequest {
    ///     user_name: "alice".to_string(),
    ///     service_name: "codecommit.amazonaws.com".to_string(),
    /// };
    ///
    /// let response = client.create_service_specific_credential(request).await?;
    /// println!("Username: {}", response.data.unwrap().service_specific_credential.service_user_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_service_specific_credential(
        &mut self,
        request: CreateServiceSpecificCredentialRequest,
    ) -> Result<AmiResponse<CreateServiceSpecificCredentialResponse>> {
        let store = self.iam_store().await?;

        // Validate user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("User {}", request.user_name),
            });
        }

        // Validate service name
        let provider = store.cloud_provider();

        // Validate service name using provider
        provider.validate_service_name(&request.service_name)?;

        // Check if user already has max credentials for this service (provider-specific limit)
        let existing = store
            .list_service_specific_credentials(
                Some(request.user_name.as_str()),
                Some(request.service_name.as_str()),
            )
            .await?;
        let max_creds = provider
            .resource_limits()
            .max_service_credentials_per_user_per_service;
        if existing.len() >= max_creds {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "User {} already has the maximum number of credentials ({}) for service {}",
                    request.user_name, max_creds, request.service_name
                ),
            });
        }

        // Use provider for credential ID generation
        let cred_id = provider.generate_resource_id(ResourceType::ServiceCredential);

        // Generate service username (format: username-at-account_id)
        let account_id = store.account_id();
        let service_user_name = format!("{}-at-{}", request.user_name, account_id);

        // Generate service password (random string)
        let service_password = uuid::Uuid::new_v4().to_string().replace('-', "");

        // Generate WAMI ARN for cross-provider identification
        let wami_arn =
            provider.generate_wami_arn(ResourceType::ServiceCredential, account_id, "/", &cred_id);

        let credential = ServiceSpecificCredential {
            user_name: request.user_name.clone(),
            service_specific_credential_id: cred_id,
            service_user_name: service_user_name.clone(),
            service_password: Some(service_password.clone()),
            service_name: request.service_name,
            create_date: Utc::now(),
            status: "Active".to_string(),
            wami_arn,
            providers: Vec::new(),
        };

        store
            .create_service_specific_credential(credential.clone())
            .await?;

        Ok(AmiResponse::success(
            CreateServiceSpecificCredentialResponse {
                service_specific_credential: credential,
            },
        ))
    }

    /// Delete a service-specific credential
    ///
    /// Deletes the specified service-specific credential.
    ///
    /// # Arguments
    ///
    /// * `request` - The delete service-specific credential request
    ///
    /// # Returns
    ///
    /// Returns success if the credential was deleted
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, DeleteServiceSpecificCredentialRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = DeleteServiceSpecificCredentialRequest {
    ///     user_name: "alice".to_string(),
    ///     service_specific_credential_id: "ACCA123...".to_string(),
    /// };
    ///
    /// let response = client.delete_service_specific_credential(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_service_specific_credential(
        &mut self,
        request: DeleteServiceSpecificCredentialRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Validate user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("User {}", request.user_name),
            });
        }

        // Check if credential exists
        if store
            .get_service_specific_credential(&request.service_specific_credential_id)
            .await?
            .is_none()
        {
            return Err(AmiError::ResourceNotFound {
                resource: format!(
                    "Service-specific credential {}",
                    request.service_specific_credential_id
                ),
            });
        }

        store
            .delete_service_specific_credential(&request.service_specific_credential_id)
            .await?;

        Ok(AmiResponse::success(()))
    }

    /// List service-specific credentials
    ///
    /// Lists the service-specific credentials for a user or all users.
    ///
    /// # Arguments
    ///
    /// * `request` - The list service-specific credentials request
    ///
    /// # Returns
    ///
    /// Returns a list of credential metadata (without passwords)
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, ListServiceSpecificCredentialsRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = ListServiceSpecificCredentialsRequest {
    ///     user_name: Some("alice".to_string()),
    ///     service_name: Some("codecommit.amazonaws.com".to_string()),
    /// };
    ///
    /// let response = client.list_service_specific_credentials(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_service_specific_credentials(
        &mut self,
        request: ListServiceSpecificCredentialsRequest,
    ) -> Result<AmiResponse<ListServiceSpecificCredentialsResponse>> {
        let store = self.iam_store().await?;

        // Validate user exists if provided
        if let Some(ref user_name) = request.user_name {
            if store.get_user(user_name).await?.is_none() {
                return Err(AmiError::ResourceNotFound {
                    resource: format!("User {}", user_name),
                });
            }
        }

        let credentials = store
            .list_service_specific_credentials(
                request.user_name.as_deref(),
                request.service_name.as_deref(),
            )
            .await?;

        // Convert to metadata (remove passwords)
        let metadata: Vec<ServiceSpecificCredentialMetadata> = credentials
            .into_iter()
            .map(|c| ServiceSpecificCredentialMetadata {
                user_name: c.user_name,
                service_specific_credential_id: c.service_specific_credential_id,
                service_user_name: c.service_user_name,
                service_name: c.service_name,
                create_date: c.create_date,
                status: c.status,
            })
            .collect();

        Ok(AmiResponse::success(
            ListServiceSpecificCredentialsResponse {
                service_specific_credentials: metadata,
            },
        ))
    }

    /// Reset a service-specific credential password
    ///
    /// Resets the password for a service-specific credential.
    ///
    /// # Arguments
    ///
    /// * `request` - The reset service-specific credential request
    ///
    /// # Returns
    ///
    /// Returns the credential with new password
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, ResetServiceSpecificCredentialRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = ResetServiceSpecificCredentialRequest {
    ///     user_name: "alice".to_string(),
    ///     service_specific_credential_id: "ACCA123...".to_string(),
    /// };
    ///
    /// let response = client.reset_service_specific_credential(request).await?;
    /// println!("New password: {:?}", response.data.unwrap().service_specific_credential.service_password);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reset_service_specific_credential(
        &mut self,
        request: ResetServiceSpecificCredentialRequest,
    ) -> Result<AmiResponse<ResetServiceSpecificCredentialResponse>> {
        let store = self.iam_store().await?;

        // Validate user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("User {}", request.user_name),
            });
        }

        // Get existing credential
        let mut credential = store
            .get_service_specific_credential(&request.service_specific_credential_id)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!(
                    "Service-specific credential {}",
                    request.service_specific_credential_id
                ),
            })?;

        // Validate credential belongs to user
        if credential.user_name != request.user_name {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Credential {} does not belong to user {}",
                    request.service_specific_credential_id, request.user_name
                ),
            });
        }

        // Generate new password
        let new_password = uuid::Uuid::new_v4().to_string().replace('-', "");
        credential.service_password = Some(new_password);

        store
            .update_service_specific_credential(credential.clone())
            .await?;

        Ok(AmiResponse::success(
            ResetServiceSpecificCredentialResponse {
                service_specific_credential: credential,
            },
        ))
    }

    /// Update a service-specific credential status
    ///
    /// Updates the status of a service-specific credential (Active or Inactive).
    ///
    /// # Arguments
    ///
    /// * `request` - The update service-specific credential request
    ///
    /// # Returns
    ///
    /// Returns success if the credential was updated
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, UpdateServiceSpecificCredentialRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = UpdateServiceSpecificCredentialRequest {
    ///     user_name: "alice".to_string(),
    ///     service_specific_credential_id: "ACCA123...".to_string(),
    ///     status: "Inactive".to_string(),
    /// };
    ///
    /// let response = client.update_service_specific_credential(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_service_specific_credential(
        &mut self,
        request: UpdateServiceSpecificCredentialRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Validate status
        if request.status != "Active" && request.status != "Inactive" {
            return Err(AmiError::InvalidParameter {
                message: "Status must be Active or Inactive".to_string(),
            });
        }

        // Validate user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("User {}", request.user_name),
            });
        }

        // Get existing credential
        let mut credential = store
            .get_service_specific_credential(&request.service_specific_credential_id)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!(
                    "Service-specific credential {}",
                    request.service_specific_credential_id
                ),
            })?;

        // Validate credential belongs to user
        if credential.user_name != request.user_name {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Credential {} does not belong to user {}",
                    request.service_specific_credential_id, request.user_name
                ),
            });
        }

        // Update status
        credential.status = request.status;
        // Clear password for storage (only returned on creation/reset)
        credential.service_password = None;

        store.update_service_specific_credential(credential).await?;

        Ok(AmiResponse::success(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::user::CreateUserRequest;

    #[tokio::test]
    async fn test_create_service_specific_credential() {
        let store = crate::store::memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user first
        client
            .create_user(CreateUserRequest {
                user_name: "alice".to_string(),
                path: None,
                permissions_boundary: None,
                tags: None,
            })
            .await
            .unwrap();

        let request = CreateServiceSpecificCredentialRequest {
            user_name: "alice".to_string(),
            service_name: "codecommit.amazonaws.com".to_string(),
        };

        let response = client
            .create_service_specific_credential(request)
            .await
            .unwrap();
        assert!(response.success);

        let cred = response.data.unwrap().service_specific_credential;
        assert_eq!(cred.user_name, "alice");
        assert_eq!(cred.service_name, "codecommit.amazonaws.com");
        assert!(cred.service_password.is_some());
        assert!(cred.service_user_name.contains("alice"));
        assert_eq!(cred.status, "Active");
    }

    #[tokio::test]
    async fn test_create_credential_limit() {
        let store = crate::store::memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user
        client
            .create_user(CreateUserRequest {
                user_name: "bob".to_string(),
                path: None,
                permissions_boundary: None,
                tags: None,
            })
            .await
            .unwrap();

        // Create 2 credentials (max)
        for _ in 0..2 {
            client
                .create_service_specific_credential(CreateServiceSpecificCredentialRequest {
                    user_name: "bob".to_string(),
                    service_name: "codecommit.amazonaws.com".to_string(),
                })
                .await
                .unwrap();
        }

        // Try to create a 3rd (should fail)
        let result = client
            .create_service_specific_credential(CreateServiceSpecificCredentialRequest {
                user_name: "bob".to_string(),
                service_name: "codecommit.amazonaws.com".to_string(),
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_service_specific_credentials() {
        let store = crate::store::memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user
        client
            .create_user(CreateUserRequest {
                user_name: "charlie".to_string(),
                path: None,
                permissions_boundary: None,
                tags: None,
            })
            .await
            .unwrap();

        // Create credentials
        client
            .create_service_specific_credential(CreateServiceSpecificCredentialRequest {
                user_name: "charlie".to_string(),
                service_name: "codecommit.amazonaws.com".to_string(),
            })
            .await
            .unwrap();

        // List credentials
        let request = ListServiceSpecificCredentialsRequest {
            user_name: Some("charlie".to_string()),
            service_name: None,
        };
        let response = client
            .list_service_specific_credentials(request)
            .await
            .unwrap();
        assert!(response.success);

        let creds = response.data.unwrap().service_specific_credentials;
        assert_eq!(creds.len(), 1);
        assert_eq!(creds[0].user_name, "charlie");
        assert_eq!(creds[0].service_name, "codecommit.amazonaws.com");
    }

    #[tokio::test]
    async fn test_delete_service_specific_credential() {
        let store = crate::store::memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user and credential
        client
            .create_user(CreateUserRequest {
                user_name: "dave".to_string(),
                path: None,
                permissions_boundary: None,
                tags: None,
            })
            .await
            .unwrap();

        let create_response = client
            .create_service_specific_credential(CreateServiceSpecificCredentialRequest {
                user_name: "dave".to_string(),
                service_name: "codecommit.amazonaws.com".to_string(),
            })
            .await
            .unwrap();
        let cred_id = create_response
            .data
            .unwrap()
            .service_specific_credential
            .service_specific_credential_id;

        // Delete credential
        let delete_request = DeleteServiceSpecificCredentialRequest {
            user_name: "dave".to_string(),
            service_specific_credential_id: cred_id.clone(),
        };
        let response = client
            .delete_service_specific_credential(delete_request)
            .await
            .unwrap();
        assert!(response.success);

        // Verify it's deleted
        let list_response = client
            .list_service_specific_credentials(ListServiceSpecificCredentialsRequest {
                user_name: Some("dave".to_string()),
                service_name: None,
            })
            .await
            .unwrap();
        assert_eq!(
            list_response
                .data
                .unwrap()
                .service_specific_credentials
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn test_reset_service_specific_credential() {
        let store = crate::store::memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user and credential
        client
            .create_user(CreateUserRequest {
                user_name: "eve".to_string(),
                path: None,
                permissions_boundary: None,
                tags: None,
            })
            .await
            .unwrap();

        let create_response = client
            .create_service_specific_credential(CreateServiceSpecificCredentialRequest {
                user_name: "eve".to_string(),
                service_name: "codecommit.amazonaws.com".to_string(),
            })
            .await
            .unwrap();
        let cred = create_response.data.unwrap().service_specific_credential;
        let old_password = cred.service_password.clone().unwrap();

        // Reset credential
        let reset_request = ResetServiceSpecificCredentialRequest {
            user_name: "eve".to_string(),
            service_specific_credential_id: cred.service_specific_credential_id,
        };
        let response = client
            .reset_service_specific_credential(reset_request)
            .await
            .unwrap();
        assert!(response.success);

        let new_cred = response.data.unwrap().service_specific_credential;
        let new_password = new_cred.service_password.unwrap();
        assert_ne!(old_password, new_password);
    }

    #[tokio::test]
    async fn test_update_service_specific_credential() {
        let store = crate::store::memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user and credential
        client
            .create_user(CreateUserRequest {
                user_name: "frank".to_string(),
                path: None,
                permissions_boundary: None,
                tags: None,
            })
            .await
            .unwrap();

        let create_response = client
            .create_service_specific_credential(CreateServiceSpecificCredentialRequest {
                user_name: "frank".to_string(),
                service_name: "codecommit.amazonaws.com".to_string(),
            })
            .await
            .unwrap();
        let cred_id = create_response
            .data
            .unwrap()
            .service_specific_credential
            .service_specific_credential_id;

        // Update to Inactive
        let update_request = UpdateServiceSpecificCredentialRequest {
            user_name: "frank".to_string(),
            service_specific_credential_id: cred_id.clone(),
            status: "Inactive".to_string(),
        };
        let response = client
            .update_service_specific_credential(update_request)
            .await
            .unwrap();
        assert!(response.success);

        // Verify status changed
        let list_response = client
            .list_service_specific_credentials(ListServiceSpecificCredentialsRequest {
                user_name: Some("frank".to_string()),
                service_name: None,
            })
            .await
            .unwrap();
        let creds = list_response.data.unwrap().service_specific_credentials;
        assert_eq!(creds[0].status, "Inactive");
    }

    #[tokio::test]
    async fn test_invalid_service_name() {
        let store = crate::store::memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user
        client
            .create_user(CreateUserRequest {
                user_name: "grace".to_string(),
                path: None,
                permissions_boundary: None,
                tags: None,
            })
            .await
            .unwrap();

        // Try with invalid service
        let request = CreateServiceSpecificCredentialRequest {
            user_name: "grace".to_string(),
            service_name: "invalid.service.com".to_string(),
        };
        let result = client.create_service_specific_credential(request).await;
        assert!(result.is_err());
    }
}
