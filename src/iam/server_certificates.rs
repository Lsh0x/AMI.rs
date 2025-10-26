//! IAM Server Certificate Management
//!
//! This module provides functionality for managing SSL/TLS server certificates
//! used with AWS services like Elastic Load Balancing and CloudFront.

use crate::error::{AmiError, Result};
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Server certificate metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCertificateMetadata {
    /// Path to the server certificate
    #[serde(rename = "Path")]
    pub path: String,

    /// Name of the server certificate
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,

    /// ARN of the server certificate
    #[serde(rename = "Arn")]
    pub arn: String,

    /// Server certificate ID
    #[serde(rename = "ServerCertificateId")]
    pub server_certificate_id: String,

    /// Date and time when the certificate was uploaded
    #[serde(rename = "UploadDate")]
    pub upload_date: DateTime<Utc>,

    /// Date and time when the certificate expires
    #[serde(rename = "Expiration", skip_serializing_if = "Option::is_none")]
    pub expiration: Option<DateTime<Utc>>,
}

/// Server certificate with body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCertificate {
    /// Certificate metadata
    #[serde(rename = "ServerCertificateMetadata")]
    pub server_certificate_metadata: ServerCertificateMetadata,

    /// Contents of the public key certificate in PEM-encoded format
    #[serde(rename = "CertificateBody")]
    pub certificate_body: String,

    /// Contents of the certificate chain in PEM-encoded format
    #[serde(rename = "CertificateChain", skip_serializing_if = "Option::is_none")]
    pub certificate_chain: Option<String>,

    /// Tags associated with the certificate
    #[serde(rename = "Tags", skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<crate::types::Tag>,
}

/// Request to upload a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadServerCertificateRequest {
    /// Name for the server certificate
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,

    /// Contents of the public key certificate in PEM-encoded format
    #[serde(rename = "CertificateBody")]
    pub certificate_body: String,

    /// Contents of the private key in PEM-encoded format
    #[serde(rename = "PrivateKey")]
    pub private_key: String,

    /// Contents of the certificate chain in PEM-encoded format
    #[serde(rename = "CertificateChain", skip_serializing_if = "Option::is_none")]
    pub certificate_chain: Option<String>,

    /// Path for the server certificate
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Tags to attach to the certificate
    #[serde(rename = "Tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<crate::types::Tag>>,
}

/// Response from uploading a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadServerCertificateResponse {
    /// Information about the uploaded certificate
    #[serde(rename = "ServerCertificateMetadata")]
    pub server_certificate_metadata: ServerCertificateMetadata,

    /// Tags attached to the certificate
    #[serde(rename = "Tags", skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<crate::types::Tag>,
}

/// Request to get a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServerCertificateRequest {
    /// Name of the server certificate to retrieve
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,
}

/// Response from getting a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServerCertificateResponse {
    /// The server certificate
    #[serde(rename = "ServerCertificate")]
    pub server_certificate: ServerCertificate,
}

/// Request to list server certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServerCertificatesRequest {
    /// Path prefix to filter certificates
    #[serde(rename = "PathPrefix", skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,

    /// Marker for pagination
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,

    /// Maximum number of items to return
    #[serde(rename = "MaxItems", skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
}

/// Response from listing server certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListServerCertificatesResponse {
    /// List of server certificate metadata
    #[serde(rename = "ServerCertificateMetadataList")]
    pub server_certificate_metadata_list: Vec<ServerCertificateMetadata>,

    /// Indicates whether the list is truncated
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,

    /// Marker for next page
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// Request to delete a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteServerCertificateRequest {
    /// Name of the server certificate to delete
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,
}

/// Request to update a server certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerCertificateRequest {
    /// Current name of the server certificate
    #[serde(rename = "ServerCertificateName")]
    pub server_certificate_name: String,

    /// New name for the server certificate
    #[serde(
        rename = "NewServerCertificateName",
        skip_serializing_if = "Option::is_none"
    )]
    pub new_server_certificate_name: Option<String>,

    /// New path for the server certificate
    #[serde(rename = "NewPath", skip_serializing_if = "Option::is_none")]
    pub new_path: Option<String>,
}

impl<S: Store> IamClient<S> {
    /// Upload a server certificate
    ///
    /// Uploads a server certificate entity for the AWS account. The server certificate
    /// can then be referenced in AWS service configurations.
    ///
    /// # Arguments
    ///
    /// * `request` - The upload server certificate request
    ///
    /// # Returns
    ///
    /// Returns metadata about the uploaded certificate
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyiam::{MemoryIamClient, UploadServerCertificateRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let cert_body = "-----BEGIN CERTIFICATE-----\nMIIC...\n-----END CERTIFICATE-----";
    /// let private_key = "-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----";
    ///
    /// let request = UploadServerCertificateRequest {
    ///     server_certificate_name: "my-cert".to_string(),
    ///     certificate_body: cert_body.to_string(),
    ///     private_key: private_key.to_string(),
    ///     certificate_chain: None,
    ///     path: Some("/cloudfront/".to_string()),
    ///     tags: None,
    /// };
    ///
    /// let response = client.upload_server_certificate(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_server_certificate(
        &mut self,
        request: UploadServerCertificateRequest,
    ) -> Result<AmiResponse<UploadServerCertificateResponse>> {
        let store = self.iam_store().await?;

        // Validate certificate name
        if request.server_certificate_name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Server certificate name cannot be empty".to_string(),
            });
        }

        // Check if certificate already exists
        if store
            .get_server_certificate(&request.server_certificate_name)
            .await?
            .is_some()
        {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Server certificate already exists: {}",
                    request.server_certificate_name
                ),
            });
        }

        // Validate certificate body format (basic check)
        if !request.certificate_body.contains("BEGIN CERTIFICATE") {
            return Err(AmiError::InvalidParameter {
                message: "Certificate body must be in PEM format".to_string(),
            });
        }

        // Validate private key format (basic check)
        if !request.private_key.contains("BEGIN") || !request.private_key.contains("PRIVATE KEY") {
            return Err(AmiError::InvalidParameter {
                message: "Private key must be in PEM format".to_string(),
            });
        }

        // Validate certificate chain if provided
        if let Some(ref chain) = request.certificate_chain {
            if !chain.contains("BEGIN CERTIFICATE") {
                return Err(AmiError::InvalidParameter {
                    message: "Certificate chain must be in PEM format".to_string(),
                });
            }
        }

        let path = request.path.unwrap_or_else(|| "/".to_string());
        let tags = request.tags.unwrap_or_default();

        // Validate path
        if !path.starts_with('/') || !path.ends_with('/') {
            return Err(AmiError::InvalidParameter {
                message: "Path must start and end with /".to_string(),
            });
        }

        // Generate certificate ID (format: ASCA + 17 random chars from UUID)
        let cert_id = format!(
            "ASCA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );

        // Generate ARN
        let account_id = store.account_id();
        let arn = format!(
            "arn:aws:iam::{}:server-certificate{}{}",
            account_id, path, request.server_certificate_name
        );

        let metadata = ServerCertificateMetadata {
            path: path.clone(),
            server_certificate_name: request.server_certificate_name.clone(),
            arn,
            server_certificate_id: cert_id,
            upload_date: Utc::now(),
            expiration: None, // Would need to parse cert to get actual expiration
        };

        let certificate = ServerCertificate {
            server_certificate_metadata: metadata.clone(),
            certificate_body: request.certificate_body,
            certificate_chain: request.certificate_chain,
            tags: tags.clone(),
        };

        store.create_server_certificate(certificate).await?;

        Ok(AmiResponse::success(UploadServerCertificateResponse {
            server_certificate_metadata: metadata,
            tags,
        }))
    }

    /// Get a server certificate
    ///
    /// Retrieves information about the specified server certificate stored in IAM.
    ///
    /// # Arguments
    ///
    /// * `request` - The get server certificate request
    ///
    /// # Returns
    ///
    /// Returns the server certificate (without the private key)
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyiam::{MemoryIamClient, GetServerCertificateRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = GetServerCertificateRequest {
    ///     server_certificate_name: "my-cert".to_string(),
    /// };
    ///
    /// let response = client.get_server_certificate(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_server_certificate(
        &mut self,
        request: GetServerCertificateRequest,
    ) -> Result<AmiResponse<GetServerCertificateResponse>> {
        let store = self.iam_store().await?;

        let certificate = store
            .get_server_certificate(&request.server_certificate_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Server certificate {}", request.server_certificate_name),
            })?;

        Ok(AmiResponse::success(GetServerCertificateResponse {
            server_certificate: certificate,
        }))
    }

    /// List server certificates
    ///
    /// Lists the server certificates stored in IAM that have the specified path prefix.
    ///
    /// # Arguments
    ///
    /// * `request` - The list server certificates request
    ///
    /// # Returns
    ///
    /// Returns a list of server certificate metadata
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyiam::{MemoryIamClient, ListServerCertificatesRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = ListServerCertificatesRequest {
    ///     path_prefix: Some("/cloudfront/".to_string()),
    ///     marker: None,
    ///     max_items: Some(100),
    /// };
    ///
    /// let response = client.list_server_certificates(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_server_certificates(
        &mut self,
        request: ListServerCertificatesRequest,
    ) -> Result<AmiResponse<ListServerCertificatesResponse>> {
        let store = self.iam_store().await?;

        let pagination = request.max_items.map(|max| crate::types::PaginationParams {
            marker: request.marker.clone(),
            max_items: Some(max),
        });

        let (certificates, is_truncated, marker) = store
            .list_server_certificates(request.path_prefix.as_deref(), pagination.as_ref())
            .await?;

        let metadata_list: Vec<ServerCertificateMetadata> = certificates
            .into_iter()
            .map(|cert| cert.server_certificate_metadata)
            .collect();

        Ok(AmiResponse::success(ListServerCertificatesResponse {
            server_certificate_metadata_list: metadata_list,
            is_truncated,
            marker,
        }))
    }

    /// Delete a server certificate
    ///
    /// Deletes the specified server certificate.
    ///
    /// # Arguments
    ///
    /// * `request` - The delete server certificate request
    ///
    /// # Returns
    ///
    /// Returns success if the certificate was deleted
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyiam::{MemoryIamClient, DeleteServerCertificateRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = DeleteServerCertificateRequest {
    ///     server_certificate_name: "my-cert".to_string(),
    /// };
    ///
    /// let response = client.delete_server_certificate(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_server_certificate(
        &mut self,
        request: DeleteServerCertificateRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Check if certificate exists
        if store
            .get_server_certificate(&request.server_certificate_name)
            .await?
            .is_none()
        {
            return Err(AmiError::ResourceNotFound {
                resource: format!("Server certificate {}", request.server_certificate_name),
            });
        }

        store
            .delete_server_certificate(&request.server_certificate_name)
            .await?;

        Ok(AmiResponse::success(()))
    }

    /// Update a server certificate
    ///
    /// Updates the name and/or path of the specified server certificate.
    ///
    /// # Arguments
    ///
    /// * `request` - The update server certificate request
    ///
    /// # Returns
    ///
    /// Returns success if the certificate was updated
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyiam::{MemoryIamClient, UpdateServerCertificateRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = UpdateServerCertificateRequest {
    ///     server_certificate_name: "my-cert".to_string(),
    ///     new_server_certificate_name: Some("my-new-cert".to_string()),
    ///     new_path: Some("/elb/".to_string()),
    /// };
    ///
    /// let response = client.update_server_certificate(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_server_certificate(
        &mut self,
        request: UpdateServerCertificateRequest,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Get existing certificate
        let mut certificate = store
            .get_server_certificate(&request.server_certificate_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Server certificate {}", request.server_certificate_name),
            })?;

        // Update name if provided
        if let Some(new_name) = request.new_server_certificate_name {
            // Check if new name already exists
            if store.get_server_certificate(&new_name).await?.is_some() {
                return Err(AmiError::InvalidParameter {
                    message: format!("Server certificate already exists: {}", new_name),
                });
            }

            // Delete old certificate
            store
                .delete_server_certificate(&request.server_certificate_name)
                .await?;

            // Update metadata
            certificate
                .server_certificate_metadata
                .server_certificate_name = new_name.clone();

            // Update ARN if name changed
            let account_id = store.account_id();
            certificate.server_certificate_metadata.arn = format!(
                "arn:aws:iam::{}:server-certificate{}{}",
                account_id, certificate.server_certificate_metadata.path, new_name
            );
        }

        // Update path if provided
        if let Some(new_path) = request.new_path {
            // Validate path
            if !new_path.starts_with('/') || !new_path.ends_with('/') {
                return Err(AmiError::InvalidParameter {
                    message: "Path must start and end with /".to_string(),
                });
            }

            certificate.server_certificate_metadata.path = new_path.clone();

            // Update ARN with new path
            let account_id = store.account_id();
            certificate.server_certificate_metadata.arn = format!(
                "arn:aws:iam::{}:server-certificate{}{}",
                account_id,
                new_path,
                certificate
                    .server_certificate_metadata
                    .server_certificate_name
            );
        }

        // Save updated certificate
        store.update_server_certificate(certificate).await?;

        Ok(AmiResponse::success(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upload_server_certificate() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        let request = UploadServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
            certificate_chain: None,
            path: Some("/cloudfront/".to_string()),
            tags: None,
        };

        let response = client.upload_server_certificate(request).await.unwrap();
        assert!(response.success);

        let metadata = response.data.unwrap().server_certificate_metadata;
        assert_eq!(metadata.server_certificate_name, "test-cert");
        assert_eq!(metadata.path, "/cloudfront/");
        assert!(metadata
            .arn
            .contains("server-certificate/cloudfront/test-cert"));
    }

    #[tokio::test]
    async fn test_upload_duplicate_certificate() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        let request = UploadServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
            certificate_chain: None,
            path: None,
            tags: None,
        };

        client
            .upload_server_certificate(request.clone())
            .await
            .unwrap();

        let result = client.upload_server_certificate(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_server_certificate() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Upload certificate first
        let upload_request = UploadServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
            certificate_chain: Some(
                "-----BEGIN CERTIFICATE-----\nchain\n-----END CERTIFICATE-----".to_string(),
            ),
            path: None,
            tags: None,
        };
        client
            .upload_server_certificate(upload_request)
            .await
            .unwrap();

        // Get certificate
        let get_request = GetServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
        };
        let response = client.get_server_certificate(get_request).await.unwrap();
        assert!(response.success);

        let cert = response.data.unwrap().server_certificate;
        assert_eq!(
            cert.server_certificate_metadata.server_certificate_name,
            "test-cert"
        );
        assert!(cert.certificate_body.contains("BEGIN CERTIFICATE"));
        assert!(cert.certificate_chain.is_some());
    }

    #[tokio::test]
    async fn test_list_server_certificates() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Upload multiple certificates
        for i in 1..=3 {
            let request = UploadServerCertificateRequest {
                server_certificate_name: format!("cert-{}", i),
                certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                    .to_string(),
                private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                    .to_string(),
                certificate_chain: None,
                path: Some("/cloudfront/".to_string()),
                tags: None,
            };
            client.upload_server_certificate(request).await.unwrap();
        }

        // List certificates
        let list_request = ListServerCertificatesRequest {
            path_prefix: Some("/cloudfront/".to_string()),
            marker: None,
            max_items: None,
        };
        let response = client.list_server_certificates(list_request).await.unwrap();
        assert!(response.success);

        let list = response.data.unwrap();
        assert_eq!(list.server_certificate_metadata_list.len(), 3);
        assert!(!list.is_truncated);
    }

    #[tokio::test]
    async fn test_delete_server_certificate() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Upload certificate
        let upload_request = UploadServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
            certificate_chain: None,
            path: None,
            tags: None,
        };
        client
            .upload_server_certificate(upload_request)
            .await
            .unwrap();

        // Delete certificate
        let delete_request = DeleteServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
        };
        let response = client
            .delete_server_certificate(delete_request)
            .await
            .unwrap();
        assert!(response.success);

        // Verify it's deleted
        let get_request = GetServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
        };
        let result = client.get_server_certificate(get_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_server_certificate() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Upload certificate
        let upload_request = UploadServerCertificateRequest {
            server_certificate_name: "old-cert".to_string(),
            certificate_body: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
            certificate_chain: None,
            path: Some("/old/".to_string()),
            tags: None,
        };
        client
            .upload_server_certificate(upload_request)
            .await
            .unwrap();

        // Update certificate
        let update_request = UpdateServerCertificateRequest {
            server_certificate_name: "old-cert".to_string(),
            new_server_certificate_name: Some("new-cert".to_string()),
            new_path: Some("/new/".to_string()),
        };
        let response = client
            .update_server_certificate(update_request)
            .await
            .unwrap();
        assert!(response.success);

        // Verify update
        let get_request = GetServerCertificateRequest {
            server_certificate_name: "new-cert".to_string(),
        };
        let response = client.get_server_certificate(get_request).await.unwrap();
        let cert = response.data.unwrap().server_certificate;
        assert_eq!(
            cert.server_certificate_metadata.server_certificate_name,
            "new-cert"
        );
        assert_eq!(cert.server_certificate_metadata.path, "/new/");
    }

    #[tokio::test]
    async fn test_invalid_certificate_format() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        let request = UploadServerCertificateRequest {
            server_certificate_name: "test-cert".to_string(),
            certificate_body: "invalid cert".to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
            certificate_chain: None,
            path: None,
            tags: None,
        };

        let result = client.upload_server_certificate(request).await;
        assert!(result.is_err());
    }
}
