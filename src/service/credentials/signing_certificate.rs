//! Signing Certificate Service
//!
//! Orchestrates signing certificate management operations.

use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::SigningCertificateStore;
use crate::wami::credentials::signing_certificate::{
    builder as cert_builder, DeleteSigningCertificateRequest, ListSigningCertificatesRequest,
    SigningCertificate, UpdateSigningCertificateRequest, UploadSigningCertificateRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM signing certificates
///
/// Provides high-level operations for X.509 certificate management.
pub struct SigningCertificateService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: SigningCertificateStore> SigningCertificateService<S> {
    /// Create a new SigningCertificateService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Upload a new signing certificate
    pub async fn upload_signing_certificate(
        &self,
        context: &WamiContext,
        request: UploadSigningCertificateRequest,
    ) -> Result<SigningCertificate> {
        // Use wami builder to create certificate
        let certificate = cert_builder::build_signing_certificate(
            request.user_name,
            request.certificate_body,
            context,
        )?;

        // Store it
        self.store
            .write()
            .unwrap()
            .create_signing_certificate(certificate)
            .await
    }

    /// Get a signing certificate by ID
    pub async fn get_signing_certificate(
        &self,
        certificate_id: &str,
    ) -> Result<Option<SigningCertificate>> {
        self.store
            .read()
            .unwrap()
            .get_signing_certificate(certificate_id)
            .await
    }

    /// Update a signing certificate (change status)
    pub async fn update_signing_certificate(
        &self,
        request: UpdateSigningCertificateRequest,
    ) -> Result<SigningCertificate> {
        // Get existing certificate
        let mut certificate = self
            .store
            .read()
            .unwrap()
            .get_signing_certificate(&request.certificate_id)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("SigningCertificate: {}", request.certificate_id),
            })?;

        // Apply updates
        certificate.status = request.status;

        // Store updated certificate
        self.store
            .write()
            .unwrap()
            .update_signing_certificate(certificate)
            .await
    }

    /// Delete a signing certificate
    pub async fn delete_signing_certificate(
        &self,
        request: DeleteSigningCertificateRequest,
    ) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_signing_certificate(&request.certificate_id)
            .await
    }

    /// List signing certificates for a user
    pub async fn list_signing_certificates(
        &self,
        request: ListSigningCertificatesRequest,
    ) -> Result<Vec<SigningCertificate>> {
        self.store
            .read()
            .unwrap()
            .list_signing_certificates(request.user_name.as_deref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::credentials::signing_certificate::CertificateStatus;

    fn setup_service() -> SigningCertificateService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        SigningCertificateService::new(store)
    }

    fn test_context() -> WamiContext {
        use crate::arn::{TenantPath, WamiArn};
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(0))
            .caller_arn(
                WamiArn::builder()
                    .service(crate::arn::Service::Iam)
                    .tenant_path(TenantPath::single(0))
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
    async fn test_upload_and_get_signing_certificate() {
        let service = setup_service();
        let context = test_context();

        let request = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
        };

        let certificate = service
            .upload_signing_certificate(&context, request)
            .await
            .unwrap();
        assert_eq!(certificate.user_name, "alice");
        assert!(!certificate.certificate_id.is_empty());

        let retrieved = service
            .get_signing_certificate(&certificate.certificate_id)
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_name, "alice");
    }

    #[tokio::test]
    async fn test_update_signing_certificate_status() {
        let service = setup_service();
        let context = test_context();

        let upload_req = UploadSigningCertificateRequest {
            user_name: "bob".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
        };
        let certificate = service
            .upload_signing_certificate(&context, upload_req)
            .await
            .unwrap();

        let update_req = UpdateSigningCertificateRequest {
            user_name: "bob".to_string(),
            certificate_id: certificate.certificate_id.clone(),
            status: CertificateStatus::Inactive,
        };
        let updated = service
            .update_signing_certificate(update_req)
            .await
            .unwrap();
        assert_eq!(updated.status, CertificateStatus::Inactive);
    }

    #[tokio::test]
    async fn test_delete_signing_certificate() {
        let service = setup_service();
        let context = test_context();

        let upload_req = UploadSigningCertificateRequest {
            user_name: "charlie".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
        };
        let certificate = service
            .upload_signing_certificate(&context, upload_req)
            .await
            .unwrap();

        let delete_req = DeleteSigningCertificateRequest {
            user_name: "charlie".to_string(),
            certificate_id: certificate.certificate_id.clone(),
        };
        service
            .delete_signing_certificate(delete_req)
            .await
            .unwrap();

        let retrieved = service
            .get_signing_certificate(&certificate.certificate_id)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_signing_certificates() {
        let service = setup_service();
        let context = test_context();

        // Upload multiple certificates for same user
        for _ in 0..3 {
            let request = UploadSigningCertificateRequest {
                user_name: "david".to_string(),
                certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                    .to_string(),
            };
            service
                .upload_signing_certificate(&context, request)
                .await
                .unwrap();
        }

        let list_request = ListSigningCertificatesRequest {
            user_name: Some("david".to_string()),
        };
        let certificates = service
            .list_signing_certificates(list_request)
            .await
            .unwrap();
        assert_eq!(certificates.len(), 3);
    }
}
