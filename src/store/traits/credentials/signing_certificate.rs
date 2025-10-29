//! Signing Certificate Store Trait

use crate::error::Result;
use crate::wami::credentials::signing_certificate::SigningCertificate;
use async_trait::async_trait;

/// Trait for signing certificate storage operations
#[async_trait]
pub trait SigningCertificateStore: Send + Sync {
    async fn create_signing_certificate(
        &mut self,
        certificate: SigningCertificate,
    ) -> Result<SigningCertificate>;

    async fn get_signing_certificate(
        &self,
        certificate_id: &str,
    ) -> Result<Option<SigningCertificate>>;

    async fn update_signing_certificate(
        &mut self,
        certificate: SigningCertificate,
    ) -> Result<SigningCertificate>;

    async fn delete_signing_certificate(&mut self, certificate_id: &str) -> Result<()>;

    async fn list_signing_certificates(
        &self,
        user_name: Option<&str>,
    ) -> Result<Vec<SigningCertificate>>;
}
