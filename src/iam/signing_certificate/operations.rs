//! Signing Certificate Operations

use super::requests::*;
use crate::error::Result;
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;

impl<S: Store> IamClient<S>
where
    S::IamStore: IamStore,
{
    /// Uploads an X.509 signing certificate for a user
    pub async fn upload_signing_certificate(
        &mut self,
        request: UploadSigningCertificateRequest,
    ) -> Result<AmiResponse<UploadSigningCertificateResponse>> {
        let account_id = self.account_id().await?;
        let provider = self.cloud_provider();

        let store = self.iam_store().await?;

        store.get_user(&request.user_name).await?.ok_or_else(|| {
            crate::error::AmiError::ResourceNotFound {
                resource: format!("User {} not found", request.user_name),
            }
        })?;

        if !request.certificate_body.contains("BEGIN CERTIFICATE")
            || !request.certificate_body.contains("END CERTIFICATE")
        {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Certificate body must be in PEM-encoded X.509 format".to_string(),
            });
        }

        let existing_certs = store
            .list_signing_certificates(Some(&request.user_name))
            .await?;
        let max_certs = provider.resource_limits().max_access_keys_per_user;
        if existing_certs.len() >= max_certs {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "User {} already has the maximum number of signing certificates ({})",
                    request.user_name, max_certs
                ),
            });
        }

        let certificate = super::builder::build_signing_certificate(
            request.user_name,
            request.certificate_body,
            provider.as_ref(),
            &account_id,
        );

        let created_cert = store.create_signing_certificate(certificate).await?;

        Ok(AmiResponse::success(UploadSigningCertificateResponse {
            certificate: created_cert,
        }))
    }

    /// Deletes a signing certificate for a user
    pub async fn delete_signing_certificate(
        &mut self,
        request: DeleteSigningCertificateRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        let cert = store
            .get_signing_certificate(&request.certificate_id)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("Certificate {} not found", request.certificate_id),
            })?;

        if cert.user_name != request.user_name {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "Certificate {} does not belong to user {}",
                    request.certificate_id, request.user_name
                ),
            });
        }

        store
            .delete_signing_certificate(&request.certificate_id)
            .await?;

        Ok(AmiResponse::success(()))
    }

    /// Lists signing certificates for a user
    pub async fn list_signing_certificates(
        &mut self,
        request: Option<ListSigningCertificatesRequest>,
    ) -> Result<AmiResponse<ListSigningCertificatesResponse>> {
        let store = self.iam_store().await?;

        let user_name = request.as_ref().and_then(|r| r.user_name.as_deref());

        if let Some(user) = user_name {
            store.get_user(user).await?.ok_or_else(|| {
                crate::error::AmiError::ResourceNotFound {
                    resource: format!("User {} not found", user),
                }
            })?;
        }

        let certificates = store.list_signing_certificates(user_name).await?;

        Ok(AmiResponse::success(ListSigningCertificatesResponse {
            certificates,
            is_truncated: false,
            marker: None,
        }))
    }

    /// Updates the status of a signing certificate
    pub async fn update_signing_certificate(
        &mut self,
        request: UpdateSigningCertificateRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        let mut cert = store
            .get_signing_certificate(&request.certificate_id)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("Certificate {} not found", request.certificate_id),
            })?;

        if cert.user_name != request.user_name {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "Certificate {} does not belong to user {}",
                    request.certificate_id, request.user_name
                ),
            });
        }

        cert.status = request.status;
        store.update_signing_certificate(cert).await?;

        Ok(AmiResponse::success(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::signing_certificate::CertificateStatus;
    use crate::iam::user::CreateUserRequest;
    use crate::store::memory::InMemoryStore;

    const TEST_CERT: &str = "-----BEGIN CERTIFICATE-----
MIICiTCCAfICCQD6m7oRw0uXOjANBgkqhkiG9w0BAQUFADCBiDELMAkGA1UEBhMC
-----END CERTIFICATE-----";

    #[tokio::test]
    async fn test_upload_signing_certificate() {
        let store = InMemoryStore::new();
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

        let request = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };

        let response = client.upload_signing_certificate(request).await.unwrap();
        assert!(response.success);

        let cert = response.data.unwrap().certificate;
        assert_eq!(cert.user_name, "alice");
        assert_eq!(cert.status, CertificateStatus::Active);
    }
}
