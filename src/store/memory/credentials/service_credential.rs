//! Service-Specific Credential Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::wami::credentials::service_credential::ServiceSpecificCredential;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::ServiceCredentialStore;
use async_trait::async_trait;

#[async_trait]
impl ServiceCredentialStore for InMemoryWamiStore {
    async fn create_service_specific_credential(
        &mut self,
        credential: ServiceSpecificCredential,
    ) -> Result<ServiceSpecificCredential> {
        self.service_specific_credentials
            .insert(
                credential.service_specific_credential_id.clone(),
                credential.clone(),
            );
        Ok(credential)
    }

    async fn get_service_specific_credential(
        &self,
        credential_id: &str,
    ) -> Result<Option<ServiceSpecificCredential>> {
        Ok(self.service_specific_credentials.get(credential_id).cloned())
    }

    async fn update_service_specific_credential(
        &mut self,
        credential: ServiceSpecificCredential,
    ) -> Result<ServiceSpecificCredential> {
        self.service_specific_credentials
            .insert(
                credential.service_specific_credential_id.clone(),
                credential.clone(),
            );
        Ok(credential)
    }

    async fn delete_service_specific_credential(
        &mut self,
        service_specific_credential_id: &str,
    ) -> Result<()> {
        self.service_specific_credentials
            .remove(service_specific_credential_id);
        Ok(())
    }

    async fn list_service_specific_credentials(
        &self,
        user_name: &str,
    ) -> Result<Vec<ServiceSpecificCredential>> {
        let creds: Vec<ServiceSpecificCredential> = self
            .service_specific_credentials
            .values()
            .filter(|cred| cred.user_name == user_name)
            .cloned()
            .collect();
        Ok(creds)
    }
}


