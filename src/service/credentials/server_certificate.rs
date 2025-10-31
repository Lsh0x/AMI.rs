//! Server Certificate Service
//!
//! Orchestrates server certificate management operations.

use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::ServerCertificateStore;
use crate::types::PaginationParams;
use crate::wami::credentials::server_certificate::{
    builder as cert_builder, ListServerCertificatesRequest, ServerCertificateMetadata,
    UpdateServerCertificateRequest, UploadServerCertificateRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM server certificates
///
/// Provides high-level operations for server certificate management.
pub struct ServerCertificateService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: ServerCertificateStore> ServerCertificateService<S> {
    /// Create a new ServerCertificateService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Upload a new server certificate
    pub async fn upload_server_certificate(
        &self,
        context: &WamiContext,
        request: UploadServerCertificateRequest,
    ) -> Result<ServerCertificateMetadata> {
        // Use wami builder to create certificate
        let certificate = cert_builder::build_server_certificate(
            request.server_certificate_name,
            request.certificate_body,
            request.certificate_chain,
            request.path.unwrap_or_else(|| "/".to_string()),
            request.tags.unwrap_or_default(),
            context,
        )?;

        // Store it (note: private_key is part of ServerCertificate, not passed separately)
        self.store
            .write()
            .unwrap()
            .create_server_certificate(certificate)
            .await
    }

    /// Get a server certificate by name
    pub async fn get_server_certificate(
        &self,
        certificate_name: &str,
    ) -> Result<Option<ServerCertificateMetadata>> {
        self.store
            .read()
            .unwrap()
            .get_server_certificate(certificate_name)
            .await
    }

    /// Update a server certificate
    pub async fn update_server_certificate(
        &self,
        request: UpdateServerCertificateRequest,
    ) -> Result<ServerCertificateMetadata> {
        // Get existing certificate
        let mut certificate = self
            .store
            .read()
            .unwrap()
            .get_server_certificate(&request.server_certificate_name)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("ServerCertificate: {}", request.server_certificate_name),
            })?;

        // Apply updates
        if let Some(new_name) = request.new_server_certificate_name {
            certificate.server_certificate_name = new_name;
        }

        if let Some(new_path) = request.new_path {
            certificate.path = new_path;
        }

        // Store updated certificate
        self.store
            .write()
            .unwrap()
            .update_server_certificate(certificate)
            .await
    }

    /// Delete a server certificate
    pub async fn delete_server_certificate(&self, certificate_name: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_server_certificate(certificate_name)
            .await
    }

    /// List server certificates with optional filtering
    pub async fn list_server_certificates(
        &self,
        request: ListServerCertificatesRequest,
    ) -> Result<(Vec<ServerCertificateMetadata>, bool, Option<String>)> {
        let pagination = if request.marker.is_some() || request.max_items.is_some() {
            Some(PaginationParams {
                marker: request.marker,
                max_items: request.max_items,
            })
        } else {
            None
        };

        self.store
            .read()
            .unwrap()
            .list_server_certificates(request.path_prefix.as_deref(), pagination.as_ref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> ServerCertificateService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        ServerCertificateService::new(store)
    }

    fn test_context() -> WamiContext {
        use crate::arn::{TenantPath, WamiArn};
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("root"))
            .caller_arn(
                WamiArn::builder()
                    .service(crate::arn::Service::Iam)
                    .tenant_path(TenantPath::single("root"))
                    .wami_instance("123456789012")
                    .resource("user", "test-user")
                    .build()
                    .unwrap(),
            )
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_upload_and_get_server_certificate() {
        let service = setup_service();
        let context = test_context();

        let request = UploadServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
            certificate_chain: None,
            path: Some("/certs/".to_string()),
            tags: None,
        };

        let metadata = service
            .upload_server_certificate(&context, request)
            .await
            .unwrap();
        assert_eq!(metadata.server_certificate_name, "test-cert");
        assert_eq!(metadata.path, "/certs/");

        let retrieved = service.get_server_certificate("test-cert").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().server_certificate_name, "test-cert");
    }

    #[tokio::test]
    async fn test_delete_server_certificate() {
        let service = setup_service();
        let context = test_context();

        let request = UploadServerCertificateRequest {
            server_certificate_name: "delete-me".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
            certificate_chain: None,
            path: None,
            tags: None,
        };
        service
            .upload_server_certificate(&context, request)
            .await
            .unwrap();

        service
            .delete_server_certificate("delete-me")
            .await
            .unwrap();

        let retrieved = service.get_server_certificate("delete-me").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_server_certificates() {
        let service = setup_service();
        let context = test_context();

        // Upload multiple certificates
        for i in 0..3 {
            let request = UploadServerCertificateRequest {
                server_certificate_name: format!("cert{}", i),
                certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                    .to_string(),
                private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                    .to_string(),
                certificate_chain: None,
                path: Some("/test/".to_string()),
                tags: None,
            };
            service
                .upload_server_certificate(&context, request)
                .await
                .unwrap();
        }

        let list_request = ListServerCertificatesRequest {
            path_prefix: Some("/test/".to_string()),
            marker: None,
            max_items: None,
        };
        let (certs, _, _) = service
            .list_server_certificates(list_request)
            .await
            .unwrap();
        assert_eq!(certs.len(), 3);
    }
}
