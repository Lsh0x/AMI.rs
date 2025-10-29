//! Server Certificate Store Trait

use crate::error::Result;
use crate::types::PaginationParams;
use crate::wami::credentials::server_certificate::ServerCertificateMetadata;
use crate::wami::credentials::ServerCertificate;
use async_trait::async_trait;

/// Trait for server certificate storage operations
#[async_trait]
pub trait ServerCertificateStore: Send + Sync {
    async fn create_server_certificate(
        &mut self,
        certificate: ServerCertificate,
    ) -> Result<ServerCertificateMetadata>;

    async fn get_server_certificate(
        &self,
        certificate_name: &str,
    ) -> Result<Option<ServerCertificateMetadata>>;

    async fn update_server_certificate(
        &mut self,
        certificate: ServerCertificateMetadata,
    ) -> Result<ServerCertificateMetadata>;

    async fn delete_server_certificate(&mut self, certificate_name: &str) -> Result<()>;

    async fn list_server_certificates(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<ServerCertificateMetadata>, bool, Option<String>)>;
}
