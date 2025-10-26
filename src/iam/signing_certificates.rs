use crate::error::Result;
use crate::iam::IamClient;
use crate::provider::ResourceType;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Signing certificate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CertificateStatus {
    /// Certificate is active
    Active,
    /// Certificate is inactive
    Inactive,
}

/// Signing certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningCertificate {
    /// The name of the user the signing certificate is associated with
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The ID for the signing certificate
    #[serde(rename = "CertificateId")]
    pub certificate_id: String,

    /// The contents of the signing certificate (PEM-encoded)
    #[serde(rename = "CertificateBody")]
    pub certificate_body: String,

    /// The status of the signing certificate
    #[serde(rename = "Status")]
    pub status: CertificateStatus,

    /// The date and time when the signing certificate was uploaded
    #[serde(rename = "UploadDate")]
    pub upload_date: DateTime<Utc>,

    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,

    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Request to upload a signing certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSigningCertificateRequest {
    /// The name of the user the signing certificate is for
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The contents of the signing certificate (PEM-encoded X.509 certificate)
    #[serde(rename = "CertificateBody")]
    pub certificate_body: String,
}

/// Response from uploading a signing certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSigningCertificateResponse {
    /// Information about the uploaded signing certificate
    #[serde(rename = "Certificate")]
    pub certificate: SigningCertificate,
}

/// Request to delete a signing certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSigningCertificateRequest {
    /// The name of the user the signing certificate belongs to
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The ID of the signing certificate to delete
    #[serde(rename = "CertificateId")]
    pub certificate_id: String,
}

/// Request to list signing certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSigningCertificatesRequest {
    /// The name of the user to list signing certificates for
    #[serde(rename = "UserName", skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// Response from listing signing certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSigningCertificatesResponse {
    /// List of signing certificates
    #[serde(rename = "Certificates")]
    pub certificates: Vec<SigningCertificate>,

    /// Whether the results are truncated
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,

    /// Marker for pagination
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// Request to update a signing certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSigningCertificateRequest {
    /// The name of the user the signing certificate belongs to
    #[serde(rename = "UserName")]
    pub user_name: String,

    /// The ID of the signing certificate to update
    #[serde(rename = "CertificateId")]
    pub certificate_id: String,

    /// The new status for the signing certificate
    #[serde(rename = "Status")]
    pub status: CertificateStatus,
}

impl<S: Store> IamClient<S>
where
    S::IamStore: IamStore,
{
    /// Uploads an X.509 signing certificate for a user
    ///
    /// Uploads an X.509 signing certificate and associates it with the specified IAM user.
    /// Some AWS services require you to use certificates to validate requests that are
    /// signed with a corresponding private key. When you upload the certificate, AWS securely
    /// stores the certificate body you upload.
    ///
    /// Each user can have up to 2 signing certificates at a time.
    ///
    /// # Arguments
    ///
    /// * `request` - The upload request containing the username and certificate body
    ///
    /// # Returns
    ///
    /// Returns the uploaded signing certificate with its metadata
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, UploadSigningCertificateRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::InMemoryStore::new();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // First create a user
    /// client.create_user(CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// }).await?;
    ///
    /// let cert_body = "-----BEGIN CERTIFICATE-----\nMIIC...example...==\n-----END CERTIFICATE-----";
    ///
    /// let request = UploadSigningCertificateRequest {
    ///     user_name: "alice".to_string(),
    ///     certificate_body: cert_body.to_string(),
    /// };
    ///
    /// let response = client.upload_signing_certificate(request).await?;
    /// let cert = response.data.unwrap().certificate;
    /// println!("Certificate ID: {}", cert.certificate_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_signing_certificate(
        &mut self,
        request: UploadSigningCertificateRequest,
    ) -> Result<AmiResponse<UploadSigningCertificateResponse>> {
        let store = self.iam_store().await?;

        // Validate user exists
        store.get_user(&request.user_name).await?.ok_or_else(|| {
            crate::error::AmiError::ResourceNotFound {
                resource: format!("User {} not found", request.user_name),
            }
        })?;

        // Validate certificate body format (basic PEM check)
        if !request.certificate_body.contains("BEGIN CERTIFICATE")
            || !request.certificate_body.contains("END CERTIFICATE")
        {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Certificate body must be in PEM-encoded X.509 format".to_string(),
            });
        }

        let provider = store.cloud_provider();

        // Check certificate limit (use same limit as access keys)
        let existing_certs = store
            .list_signing_certificates(Some(&request.user_name))
            .await?;
        let max_certs = provider.resource_limits().max_access_keys_per_user; // Reuse access key limit (typically 2)
        if existing_certs.len() >= max_certs {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "User {} already has the maximum number of signing certificates ({})",
                    request.user_name, max_certs
                ),
            });
        }

        // Use provider for certificate ID generation
        let certificate_id = provider.generate_resource_id(ResourceType::SigningCertificate);

        // Get account ID for WAMI ARN generation
        let account_id = store.account_id();

        // Generate WAMI ARN for cross-provider identification
        let wami_arn = provider.generate_wami_arn(
            ResourceType::SigningCertificate,
            account_id,
            "/",
            &certificate_id,
        );

        let certificate = SigningCertificate {
            user_name: request.user_name.clone(),
            certificate_id,
            certificate_body: request.certificate_body,
            status: CertificateStatus::Active,
            upload_date: Utc::now(),
            wami_arn,
            providers: Vec::new(),
        };

        let created_cert = store.create_signing_certificate(certificate).await?;

        Ok(AmiResponse::success(UploadSigningCertificateResponse {
            certificate: created_cert,
        }))
    }

    /// Deletes a signing certificate for a user
    ///
    /// Deletes the specified signing certificate associated with the specified IAM user.
    ///
    /// # Arguments
    ///
    /// * `request` - The deletion request containing the username and certificate ID
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, DeleteSigningCertificateRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = wami::InMemoryStore::new();
    /// # let mut client = MemoryIamClient::new(store);
    /// # let certificate_id = "ASCA1234567890ABCDEF".to_string();
    /// let request = DeleteSigningCertificateRequest {
    ///     user_name: "alice".to_string(),
    ///     certificate_id,
    /// };
    ///
    /// let response = client.delete_signing_certificate(request).await?;
    /// assert!(response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_signing_certificate(
        &mut self,
        request: DeleteSigningCertificateRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Verify the certificate exists and belongs to the user
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
    ///
    /// Returns information about the signing certificates associated with the specified IAM user.
    /// If no user is specified, lists all signing certificates in the account.
    ///
    /// # Arguments
    ///
    /// * `request` - Optional request specifying the username to filter by
    ///
    /// # Returns
    ///
    /// Returns a list of signing certificates
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, ListSigningCertificatesRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = wami::InMemoryStore::new();
    /// # let mut client = MemoryIamClient::new(store);
    /// let request = ListSigningCertificatesRequest {
    ///     user_name: Some("alice".to_string()),
    /// };
    ///
    /// let response = client.list_signing_certificates(Some(request)).await?;
    /// let certs = response.data.unwrap().certificates;
    /// println!("Found {} certificates", certs.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_signing_certificates(
        &mut self,
        request: Option<ListSigningCertificatesRequest>,
    ) -> Result<AmiResponse<ListSigningCertificatesResponse>> {
        let store = self.iam_store().await?;

        let user_name = request.as_ref().and_then(|r| r.user_name.as_deref());

        // If user is specified, validate they exist
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
    ///
    /// Changes the status of the specified signing certificate from active to inactive,
    /// or vice versa. This action can be used to disable a signing certificate without
    /// deleting it, so that it can be re-enabled later.
    ///
    /// # Arguments
    ///
    /// * `request` - The update request containing the username, certificate ID, and new status
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, UpdateSigningCertificateRequest, CertificateStatus};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = wami::InMemoryStore::new();
    /// # let mut client = MemoryIamClient::new(store);
    /// # let certificate_id = "ASCA1234567890ABCDEF".to_string();
    /// let request = UpdateSigningCertificateRequest {
    ///     user_name: "alice".to_string(),
    ///     certificate_id,
    ///     status: CertificateStatus::Inactive,
    /// };
    ///
    /// let response = client.update_signing_certificate(request).await?;
    /// assert!(response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_signing_certificate(
        &mut self,
        request: UpdateSigningCertificateRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Verify the certificate exists and belongs to the user
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

        // Update the status
        cert.status = request.status;

        store.update_signing_certificate(cert).await?;

        Ok(AmiResponse::success(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::user::CreateUserRequest;
    use crate::store::memory::InMemoryStore;

    fn create_test_client() -> IamClient<InMemoryStore> {
        let store = InMemoryStore::new();
        IamClient::new(store)
    }

    async fn create_test_user(client: &mut IamClient<InMemoryStore>, username: &str) {
        let request = CreateUserRequest {
            user_name: username.to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(request).await.unwrap();
    }

    const TEST_CERT: &str = "-----BEGIN CERTIFICATE-----
MIICiTCCAfICCQD6m7oRw0uXOjANBgkqhkiG9w0BAQUFADCBiDELMAkGA1UEBhMC
VVMxCzAJBgNVBAgTAkNBMRYwFAYDVQQHEw1TYW4gRnJhbmNpc2NvMRMwEQYDVQQK
EwpFeGFtcGxlIENvMRcwFQYDVQQLEw5FeGFtcGxlIFVuaXQxEjAQBgNVBAMTCWxv
Y2FsaG9zdDESMBAGCSqGSIb3DQEJARYDQ0ExIDAeBgkqhkiG9w0BCQEWEWNhQGV4
YW1wbGUuY29tMB4XDTE2MDExMzIyMDgwNVoXDTE3MDExMjIyMDgwNVowgYgxCzAJ
BgNVBAYTAlVTMQswCQYDVQQIEwJDQTEWMBQGA1UEBxMNU2FuIEZyYW5jaXNjbzET
MBEGA1UEChMKRXhhbXBsZSBDbzEXMBUGA1UECxMORXhhbXBsZSBVbml0MRIwEAYD
VQQDEwlsb2NhbGhvc3QxEjAQBgkqhkiG9w0BCQEWA0NBMSAwHgYJKoZIhvcNAQkB
FhFjYUBleGFtcGxlLmNvbTCBnzANBgkqhkiG9w0BAQEFAAOBjQAwgYkCgYEA2pjR
z6I9T+TKqAp2p9W5QlCCwlz9xwW7qKEkIBkBMPrjL9lOqJJBN8QLXu+cN7TZJqJH
0N8nNnXQ6m7p/XqJK4u0X9c9IIRtUoLJhN4K0A7VVXmJ4jLCEECHHOp3Cg+Vhszl
bYm0/3w7g3H8wQ0e2FIJbA8bOHVTGkCKTbp8xL0CAwEAATANBgkqhkiG9w0BAQUF
AAOBgQATXvqAm0h7U7yxSUf4o5Y8lBKw6vJZN9yKKIVLZ9J2lKJLLdKjDMKwRVPR
sGKL2OTBqB7MhKXV8q7pD7Y6TH1T4Q5N2b6aVKVxBJ5qK1vK8LbJKLYG0N6pYqXa
C1GVnEJLHJBXD/R2Gz8sKmPP3oKo8mQbLfxKJ4VXRBjKJ5I8Cw==
-----END CERTIFICATE-----";

    #[tokio::test]
    async fn test_upload_signing_certificate() {
        let mut client = create_test_client();
        create_test_user(&mut client, "alice").await;

        let request = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };

        let response = client.upload_signing_certificate(request).await.unwrap();
        assert!(response.success);

        let cert = response.data.unwrap().certificate;
        assert_eq!(cert.user_name, "alice");
        assert!(cert.certificate_id.starts_with("ASCA"));
        assert_eq!(cert.status, CertificateStatus::Active);
        assert_eq!(cert.certificate_body, TEST_CERT);
    }

    #[tokio::test]
    async fn test_upload_certificate_user_not_found() {
        let mut client = create_test_client();

        let request = UploadSigningCertificateRequest {
            user_name: "nonexistent".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };

        let result = client.upload_signing_certificate(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_upload_certificate_invalid_format() {
        let mut client = create_test_client();
        create_test_user(&mut client, "alice").await;

        let request = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: "not a valid certificate".to_string(),
        };

        let result = client.upload_signing_certificate(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_certificate_limit() {
        let mut client = create_test_client();
        create_test_user(&mut client, "alice").await;

        // Upload first certificate
        let request1 = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        client.upload_signing_certificate(request1).await.unwrap();

        // Upload second certificate
        let request2 = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        client.upload_signing_certificate(request2).await.unwrap();

        // Third certificate should fail
        let request3 = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        let result = client.upload_signing_certificate(request3).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_signing_certificate() {
        let mut client = create_test_client();
        create_test_user(&mut client, "alice").await;

        // Upload a certificate
        let upload_request = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        let upload_response = client
            .upload_signing_certificate(upload_request)
            .await
            .unwrap();
        let cert_id = upload_response.data.unwrap().certificate.certificate_id;

        // Delete the certificate
        let delete_request = DeleteSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_id: cert_id.clone(),
        };
        let delete_response = client
            .delete_signing_certificate(delete_request)
            .await
            .unwrap();
        assert!(delete_response.success);

        // Verify it's deleted
        let list_response = client
            .list_signing_certificates(Some(ListSigningCertificatesRequest {
                user_name: Some("alice".to_string()),
            }))
            .await
            .unwrap();
        let certs = list_response.data.unwrap().certificates;
        assert_eq!(certs.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_certificate_wrong_user() {
        let mut client = create_test_client();
        create_test_user(&mut client, "alice").await;
        create_test_user(&mut client, "bob").await;

        // Upload a certificate for alice
        let upload_request = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        let upload_response = client
            .upload_signing_certificate(upload_request)
            .await
            .unwrap();
        let cert_id = upload_response.data.unwrap().certificate.certificate_id;

        // Try to delete it as bob (should fail)
        let delete_request = DeleteSigningCertificateRequest {
            user_name: "bob".to_string(),
            certificate_id: cert_id,
        };
        let result = client.delete_signing_certificate(delete_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_signing_certificates() {
        let mut client = create_test_client();
        create_test_user(&mut client, "alice").await;
        create_test_user(&mut client, "bob").await;

        // Upload certificates for alice
        let request1 = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        client.upload_signing_certificate(request1).await.unwrap();

        let request2 = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        client.upload_signing_certificate(request2).await.unwrap();

        // Upload certificate for bob
        let request3 = UploadSigningCertificateRequest {
            user_name: "bob".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        client.upload_signing_certificate(request3).await.unwrap();

        // List alice's certificates
        let list_response = client
            .list_signing_certificates(Some(ListSigningCertificatesRequest {
                user_name: Some("alice".to_string()),
            }))
            .await
            .unwrap();
        let certs = list_response.data.unwrap().certificates;
        assert_eq!(certs.len(), 2);
        assert!(certs.iter().all(|c| c.user_name == "alice"));

        // List all certificates
        let list_all_response = client.list_signing_certificates(None).await.unwrap();
        let all_certs = list_all_response.data.unwrap().certificates;
        assert_eq!(all_certs.len(), 3);
    }

    #[tokio::test]
    async fn test_update_signing_certificate() {
        let mut client = create_test_client();
        create_test_user(&mut client, "alice").await;

        // Upload a certificate
        let upload_request = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        let upload_response = client
            .upload_signing_certificate(upload_request)
            .await
            .unwrap();
        let cert_id = upload_response.data.unwrap().certificate.certificate_id;

        // Update status to Inactive
        let update_request = UpdateSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_id: cert_id.clone(),
            status: CertificateStatus::Inactive,
        };
        let update_response = client
            .update_signing_certificate(update_request)
            .await
            .unwrap();
        assert!(update_response.success);

        // Verify the status changed
        let list_response = client
            .list_signing_certificates(Some(ListSigningCertificatesRequest {
                user_name: Some("alice".to_string()),
            }))
            .await
            .unwrap();
        let certs = list_response.data.unwrap().certificates;
        assert_eq!(certs[0].status, CertificateStatus::Inactive);
    }

    #[tokio::test]
    async fn test_update_certificate_wrong_user() {
        let mut client = create_test_client();
        create_test_user(&mut client, "alice").await;
        create_test_user(&mut client, "bob").await;

        // Upload a certificate for alice
        let upload_request = UploadSigningCertificateRequest {
            user_name: "alice".to_string(),
            certificate_body: TEST_CERT.to_string(),
        };
        let upload_response = client
            .upload_signing_certificate(upload_request)
            .await
            .unwrap();
        let cert_id = upload_response.data.unwrap().certificate.certificate_id;

        // Try to update it as bob (should fail)
        let update_request = UpdateSigningCertificateRequest {
            user_name: "bob".to_string(),
            certificate_id: cert_id,
            status: CertificateStatus::Inactive,
        };
        let result = client.update_signing_certificate(update_request).await;
        assert!(result.is_err());
    }
}
