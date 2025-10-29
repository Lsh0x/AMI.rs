//! Service-Specific Credential Store Trait

use crate::error::Result;
use crate::wami::credentials::service_credential::ServiceSpecificCredential;
use async_trait::async_trait;

/// Trait for service-specific credential storage operations
#[async_trait]
pub trait ServiceCredentialStore: Send + Sync {
    async fn create_service_specific_credential(
        &mut self,
        credential: ServiceSpecificCredential,
    ) -> Result<ServiceSpecificCredential>;

    async fn get_service_specific_credential(
        &self,
        credential_id: &str,
    ) -> Result<Option<ServiceSpecificCredential>>;

    async fn update_service_specific_credential(
        &mut self,
        credential: ServiceSpecificCredential,
    ) -> Result<ServiceSpecificCredential>;

    async fn delete_service_specific_credential(
        &mut self,
        service_specific_credential_id: &str,
    ) -> Result<()>;

    async fn list_service_specific_credentials(
        &self,
        user_name: &str,
    ) -> Result<Vec<ServiceSpecificCredential>>;
}
