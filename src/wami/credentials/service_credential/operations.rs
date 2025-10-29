//! Service-Specific Credential Domain Operations
//!
//! Pure business logic functions for service-specific credential management.

use super::{builder, model::*, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;

/// Pure domain operations for service-specific credentials
pub mod service_credential_operations {
    use super::*;

    /// Build a new service credential from a request (pure function)
    pub fn build_from_request(
        request: CreateServiceSpecificCredentialRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
    ) -> ServiceSpecificCredential {
        builder::build_service_credential(
            request.user_name,
            request.service_name,
            provider,
            account_id,
        )
    }

    /// Apply a status update to an existing service credential (pure function)
    pub fn apply_status_update(
        mut credential: ServiceSpecificCredential,
        new_status: String,
    ) -> ServiceSpecificCredential {
        credential.status = new_status;
        credential
    }

    /// Validate service name (pure function)
    pub fn validate_service_name(service_name: &str) -> Result<()> {
        if service_name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Service name cannot be empty".to_string(),
            });
        }

        // Validate service name format
        let valid_services = [
            "codecommit.amazonaws.com",
            "cassandra.amazonaws.com",
            "iot.amazonaws.com",
        ];

        if !valid_services.contains(&service_name) {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid service name: {}. Must be one of: {}",
                    service_name,
                    valid_services.join(", ")
                ),
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
    fn test_validate_service_name() {
        assert!(
            service_credential_operations::validate_service_name("codecommit.amazonaws.com")
                .is_ok()
        );
        assert!(service_credential_operations::validate_service_name("invalid.service").is_err());
        assert!(service_credential_operations::validate_service_name("").is_err());
    }

    #[test]
    fn test_validate_status() {
        assert!(service_credential_operations::validate_status("Active").is_ok());
        assert!(service_credential_operations::validate_status("Inactive").is_ok());
        assert!(service_credential_operations::validate_status("Invalid").is_err());
    }
}
