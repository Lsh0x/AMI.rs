//! Server Certificate Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::ServerCertificateStore;
use crate::types::PaginationParams;
use crate::wami::credentials::server_certificate::ServerCertificateMetadata;
use crate::wami::credentials::ServerCertificate;
use async_trait::async_trait;

#[async_trait]
impl ServerCertificateStore for InMemoryWamiStore {
    async fn create_server_certificate(
        &mut self,
        certificate: ServerCertificate,
    ) -> Result<ServerCertificateMetadata> {
        let metadata = certificate.server_certificate_metadata.clone();
        self.server_certificates.insert(
            certificate
                .server_certificate_metadata
                .server_certificate_name
                .clone(),
            certificate,
        );
        Ok(metadata)
    }

    async fn get_server_certificate(
        &self,
        certificate_name: &str,
    ) -> Result<Option<ServerCertificateMetadata>> {
        Ok(self
            .server_certificates
            .get(certificate_name)
            .map(|cert| cert.server_certificate_metadata.clone()))
    }

    async fn update_server_certificate(
        &mut self,
        certificate: ServerCertificateMetadata,
    ) -> Result<ServerCertificateMetadata> {
        if let Some(existing) = self
            .server_certificates
            .get_mut(&certificate.server_certificate_name)
        {
            existing.server_certificate_metadata = certificate.clone();
            Ok(certificate)
        } else {
            Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Server certificate {}", certificate.server_certificate_name),
            })
        }
    }

    async fn delete_server_certificate(&mut self, certificate_name: &str) -> Result<()> {
        self.server_certificates.remove(certificate_name);
        Ok(())
    }

    async fn list_server_certificates(
        &self,
        path_prefix: Option<&str>,
        _pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<ServerCertificateMetadata>, bool, Option<String>)> {
        let certs: Vec<ServerCertificateMetadata> = self
            .server_certificates
            .values()
            .filter(|cert| {
                path_prefix
                    .is_none_or(|prefix| cert.server_certificate_metadata.path.starts_with(prefix))
            })
            .map(|cert| cert.server_certificate_metadata.clone())
            .collect();
        Ok((certs, false, None))
    }
}
