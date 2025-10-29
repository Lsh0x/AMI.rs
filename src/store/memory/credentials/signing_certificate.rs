//! Signing Certificate Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::wami::credentials::signing_certificate::SigningCertificate;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::SigningCertificateStore;
use async_trait::async_trait;

#[async_trait]
impl SigningCertificateStore for InMemoryWamiStore {
    async fn create_signing_certificate(
        &mut self,
        certificate: SigningCertificate,
    ) -> Result<SigningCertificate> {
        self.signing_certificates
            .insert(certificate.certificate_id.clone(), certificate.clone());
        Ok(certificate)
    }

    async fn get_signing_certificate(
        &self,
        certificate_id: &str,
    ) -> Result<Option<SigningCertificate>> {
        Ok(self.signing_certificates.get(certificate_id).cloned())
    }

    async fn update_signing_certificate(
        &mut self,
        certificate: SigningCertificate,
    ) -> Result<SigningCertificate> {
        self.signing_certificates
            .insert(certificate.certificate_id.clone(), certificate.clone());
        Ok(certificate)
    }

    async fn delete_signing_certificate(&mut self, certificate_id: &str) -> Result<()> {
        self.signing_certificates.remove(certificate_id);
        Ok(())
    }

    async fn list_signing_certificates(
        &self,
        user_name: Option<&str>,
    ) -> Result<Vec<SigningCertificate>> {
        let certs: Vec<SigningCertificate> = self
            .signing_certificates
            .values()
            .filter(|cert| {
                user_name.map_or(true, |name| cert.user_name == name)
            })
            .cloned()
            .collect();
        Ok(certs)
    }
}


