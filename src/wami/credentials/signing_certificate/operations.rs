//! Signing Certificate Domain Operations
//!
//! Pure business logic functions for signing certificate management.

use super::{builder, model::*, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;

/// Pure domain operations for signing certificates
pub mod signing_certificate_operations {
    use super::*;

    /// Build a new signing certificate from a request (pure function)
    pub fn build_from_request(
        request: UploadSigningCertificateRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> SigningCertificate {
        builder::build_signing_certificate(
            request.user_name,
            request.certificate_body,
            provider,
            account_id,
        )
    }

    /// Apply a status update to an existing signing certificate (pure function)
    pub fn apply_status_update(
        mut cert: SigningCertificate,
        new_status: String,
    ) -> SigningCertificate {
        cert.status = new_status;
        cert
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

    /// Validate status value (pure function)
    pub fn validate_status(status: &str) -> Result<()> {
        match status {
            "Active" | "Inactive" => Ok(()),
            _ => Err(AmiError::InvalidParameter {
                message: format!("Invalid status: {}. Must be 'Active' or 'Inactive'", status),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_certificate_body() {
        let valid_cert = "-----BEGIN CERTIFICATE-----\nMIIC...\n-----END CERTIFICATE-----";
        assert!(signing_certificate_operations::validate_certificate_body(valid_cert).is_ok());

        let invalid_cert = "not a certificate";
        assert!(signing_certificate_operations::validate_certificate_body(invalid_cert).is_err());
    }

    #[test]
    fn test_validate_status() {
        assert!(signing_certificate_operations::validate_status("Active").is_ok());
        assert!(signing_certificate_operations::validate_status("Inactive").is_ok());
        assert!(signing_certificate_operations::validate_status("Invalid").is_err());
    }
}
