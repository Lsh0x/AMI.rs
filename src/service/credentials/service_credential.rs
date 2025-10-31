//! Service Credential Service
//!
//! Orchestrates service-specific credential management operations.

use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::ServiceCredentialStore;
use crate::wami::credentials::service_credential::{
    builder as cred_builder, CreateServiceSpecificCredentialRequest,
    DeleteServiceSpecificCredentialRequest, ListServiceSpecificCredentialsRequest,
    ServiceSpecificCredential, UpdateServiceSpecificCredentialRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM service-specific credentials
///
/// Provides high-level operations for AWS service credentials (e.g., CodeCommit).
pub struct ServiceCredentialService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: ServiceCredentialStore> ServiceCredentialService<S> {
    /// Create a new ServiceCredentialService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Create a new service-specific credential
    pub async fn create_service_specific_credential(
        &self,
        context: &WamiContext,
        request: CreateServiceSpecificCredentialRequest,
    ) -> Result<ServiceSpecificCredential> {
        // Use wami builder to create credential
        let credential = cred_builder::build_service_credential(
            request.user_name,
            request.service_name,
            context,
        )?;

        // Store it
        self.store
            .write()
            .unwrap()
            .create_service_specific_credential(credential)
            .await
    }

    /// Get a service-specific credential by ID
    pub async fn get_service_specific_credential(
        &self,
        credential_id: &str,
    ) -> Result<Option<ServiceSpecificCredential>> {
        self.store
            .read()
            .unwrap()
            .get_service_specific_credential(credential_id)
            .await
    }

    /// Update a service-specific credential status
    pub async fn update_service_specific_credential(
        &self,
        request: UpdateServiceSpecificCredentialRequest,
    ) -> Result<ServiceSpecificCredential> {
        // Get existing credential
        let mut credential = self
            .store
            .read()
            .unwrap()
            .get_service_specific_credential(&request.service_specific_credential_id)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!(
                    "ServiceSpecificCredential: {}",
                    request.service_specific_credential_id
                ),
            })?;

        // Apply updates
        credential.status = request.status;

        // Store updated credential
        self.store
            .write()
            .unwrap()
            .update_service_specific_credential(credential)
            .await
    }

    /// Delete a service-specific credential
    pub async fn delete_service_specific_credential(
        &self,
        request: DeleteServiceSpecificCredentialRequest,
    ) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_service_specific_credential(&request.service_specific_credential_id)
            .await
    }

    /// List service-specific credentials for a user
    pub async fn list_service_specific_credentials(
        &self,
        request: ListServiceSpecificCredentialsRequest,
    ) -> Result<Vec<ServiceSpecificCredential>> {
        let user_name = request.user_name.as_deref().unwrap_or("");
        self.store
            .read()
            .unwrap()
            .list_service_specific_credentials(user_name)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> ServiceCredentialService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        ServiceCredentialService::new(store)
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
    async fn test_create_and_get_service_credential() {
        let service = setup_service();

        let request = CreateServiceSpecificCredentialRequest {
            user_name: "alice".to_string(),
            service_name: "codecommit.amazonaws.com".to_string(),
        };

        let context = test_context();
        let credential = service
            .create_service_specific_credential(&context, request)
            .await
            .unwrap();
        assert_eq!(credential.user_name, "alice");
        assert_eq!(credential.service_name, "codecommit.amazonaws.com");

        let retrieved = service
            .get_service_specific_credential(&credential.service_specific_credential_id)
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_name, "alice");
    }

    #[tokio::test]
    async fn test_update_service_credential_status() {
        let service = setup_service();

        let create_req = CreateServiceSpecificCredentialRequest {
            user_name: "bob".to_string(),
            service_name: "codecommit.amazonaws.com".to_string(),
        };
        let context = test_context();
        let credential = service
            .create_service_specific_credential(&context, create_req)
            .await
            .unwrap();

        let update_req = UpdateServiceSpecificCredentialRequest {
            user_name: "bob".to_string(),
            service_specific_credential_id: credential.service_specific_credential_id.clone(),
            status: "Inactive".to_string(),
        };
        let updated = service
            .update_service_specific_credential(update_req)
            .await
            .unwrap();
        assert_eq!(updated.status, "Inactive");
    }

    #[tokio::test]
    async fn test_delete_service_credential() {
        let service = setup_service();

        let create_req = CreateServiceSpecificCredentialRequest {
            user_name: "charlie".to_string(),
            service_name: "codecommit.amazonaws.com".to_string(),
        };
        let context = test_context();
        let credential = service
            .create_service_specific_credential(&context, create_req)
            .await
            .unwrap();

        let delete_req = DeleteServiceSpecificCredentialRequest {
            user_name: "charlie".to_string(),
            service_specific_credential_id: credential.service_specific_credential_id.clone(),
        };
        service
            .delete_service_specific_credential(delete_req)
            .await
            .unwrap();

        let retrieved = service
            .get_service_specific_credential(&credential.service_specific_credential_id)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_service_credentials() {
        let service = setup_service();

        // Create multiple credentials for same user
        let context = test_context();
        for _ in 0..3 {
            let request = CreateServiceSpecificCredentialRequest {
                user_name: "david".to_string(),
                service_name: "codecommit.amazonaws.com".to_string(),
            };
            service
                .create_service_specific_credential(&context, request)
                .await
                .unwrap();
        }

        let list_request = ListServiceSpecificCredentialsRequest {
            user_name: Some("david".to_string()),
            service_name: None,
        };
        let credentials = service
            .list_service_specific_credentials(list_request)
            .await
            .unwrap();
        assert_eq!(credentials.len(), 3);
    }
}
