//! Server Certificate Domain Operations
//!
//! Pure business logic functions for server certificate management.

use super::{builder, model::*, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;

/// Pure domain operations for server certificates
pub mod server_certificate_operations {
    use super::*;

    /// Build a new server certificate from a request (pure function)
    pub fn build_from_request(
        request: UploadServerCertificateRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> ServerCertificate {
        builder::build_server_certificate(
            request.server_certificate_name,
            request.certificate_body,
            request.private_key,
            request.certificate_chain,
            request.path,
            request.tags,
            provider,
            account_id,
        )
    }

    /// Validate certificate name (pure function)
    pub fn validate_certificate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Server certificate name cannot be empty".to_string(),
            });
        }

        if name.len() > 128 {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Server certificate name '{}' exceeds maximum length of 128 characters",
                    name
                ),
            });
        }

        // Validate name format (alphanumeric, hyphen, underscore)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AmiError::InvalidParameter {
                message: format!("Server certificate name '{}' contains invalid characters", name),
            });
        }

        Ok(())
    }

    /// Validate certificate body format (pure function)
    pub fn validate_certificate_body(cert_body: &str) -> Result<()> {
        if !cert_body.contains("-----BEGIN CERTIFICATE-----") {
            return Err(AmiError::InvalidParameter {
                message: "Invalid certificate body format: missing BEGIN CERTIFICATE marker"
                    .to_string(),
            });
        }

        if !cert_body.contains("-----END CERTIFICATE-----") {
                return Err(AmiError::InvalidParameter {
                message: "Invalid certificate body format: missing END CERTIFICATE marker"
                    .to_string(),
            });
        }

        Ok(())
    }

    /// Validate private key format (pure function)
    pub fn validate_private_key(private_key: &str) -> Result<()> {
        let has_rsa_begin = private_key.contains("-----BEGIN RSA PRIVATE KEY-----");
        let has_private_begin = private_key.contains("-----BEGIN PRIVATE KEY-----");

        if !has_rsa_begin && !has_private_begin {
            return Err(AmiError::InvalidParameter {
                message: "Invalid private key format: missing BEGIN PRIVATE KEY marker".to_string(),
            });
        }

        let has_rsa_end = private_key.contains("-----END RSA PRIVATE KEY-----");
        let has_private_end = private_key.contains("-----END PRIVATE KEY-----");

        if !has_rsa_end && !has_private_end {
                return Err(AmiError::InvalidParameter {
                message: "Invalid private key format: missing END PRIVATE KEY marker".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_certificate_name() {
        assert!(server_certificate_operations::validate_certificate_name("my-cert").is_ok());
        assert!(server_certificate_operations::validate_certificate_name("cert_123").is_ok());
        assert!(server_certificate_operations::validate_certificate_name("").is_err());
        assert!(server_certificate_operations::validate_certificate_name("cert@123").is_err());
    }

    #[test]
    fn test_validate_certificate_body() {
        let valid_cert = "-----BEGIN CERTIFICATE-----\nMIIC...\n-----END CERTIFICATE-----";
        assert!(server_certificate_operations::validate_certificate_body(valid_cert).is_ok());

        let invalid_cert = "not a certificate";
        assert!(server_certificate_operations::validate_certificate_body(invalid_cert).is_err());
    }

    #[test]
    fn test_validate_private_key() {
        let valid_key = "-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----";
        assert!(server_certificate_operations::validate_private_key(valid_key).is_ok());

        let invalid_key = "not a key";
        assert!(server_certificate_operations::validate_private_key(invalid_key).is_err());
    }
}
