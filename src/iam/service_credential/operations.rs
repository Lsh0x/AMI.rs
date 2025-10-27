//! Service Credential Operations

use super::model::*;
use super::requests::*;
use crate::error::{AmiError, Result};
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;

impl<S: Store> IamClient<S> {
    /// Create a service-specific credential
    pub async fn create_service_specific_credential(
        &mut self,
        request: CreateServiceSpecificCredentialRequest,
    ) -> Result<AmiResponse<CreateServiceSpecificCredentialResponse>> {
        let account_id = self.account_id().await?;
        let provider = self.cloud_provider();

        provider
            .as_ref()
            .validate_service_name(&request.service_name)?;

        let store = self.iam_store().await?;

        // Validate user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("User {}", request.user_name),
            });
        }

        // Check if user already has max credentials for this service
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

        let credential = super::builder::build_service_specific_credential(
            request.user_name,
            request.service_name,
            provider.as_ref(),
            &account_id,
        );

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
    pub async fn delete_service_specific_credential(
        &mut self,
        request: DeleteServiceSpecificCredentialRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        if store.get_user(&request.user_name).await?.is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("User {}", request.user_name),
            });
        }

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
    pub async fn list_service_specific_credentials(
        &mut self,
        request: ListServiceSpecificCredentialsRequest,
    ) -> Result<AmiResponse<ListServiceSpecificCredentialsResponse>> {
        let store = self.iam_store().await?;

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
    pub async fn reset_service_specific_credential(
        &mut self,
        request: ResetServiceSpecificCredentialRequest,
    ) -> Result<AmiResponse<ResetServiceSpecificCredentialResponse>> {
        let store = self.iam_store().await?;

        if store.get_user(&request.user_name).await?.is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("User {}", request.user_name),
            });
        }

        let mut credential = store
            .get_service_specific_credential(&request.service_specific_credential_id)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!(
                    "Service-specific credential {}",
                    request.service_specific_credential_id
                ),
            })?;

        if credential.user_name != request.user_name {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Credential {} does not belong to user {}",
                    request.service_specific_credential_id, request.user_name
                ),
            });
        }

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
    pub async fn update_service_specific_credential(
        &mut self,
        request: UpdateServiceSpecificCredentialRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        if request.status != "Active" && request.status != "Inactive" {
            return Err(AmiError::InvalidParameter {
                message: "Status must be Active or Inactive".to_string(),
            });
        }

        if store.get_user(&request.user_name).await?.is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("User {}", request.user_name),
            });
        }

        let mut credential = store
            .get_service_specific_credential(&request.service_specific_credential_id)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!(
                    "Service-specific credential {}",
                    request.service_specific_credential_id
                ),
            })?;

        if credential.user_name != request.user_name {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Credential {} does not belong to user {}",
                    request.service_specific_credential_id, request.user_name
                ),
            });
        }

        credential.status = request.status;
        credential.service_password = None;

        store.update_service_specific_credential(credential).await?;

        Ok(AmiResponse::success(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::user::CreateUserRequest;

    #[tokio::test]
    async fn test_create_service_specific_credential() {
        let store = crate::store::memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

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
        assert_eq!(cred.status, "Active");
    }
}
