//! Identity Provider Operations
//!
//! Pure validation and parsing functions for SAML and OIDC providers.

#![allow(clippy::result_large_err)]

use crate::error::{AmiError, Result};
use chrono::{DateTime, Utc};

/// Validate SAML metadata document (XML format)
///
/// Performs basic XML validation and checks for required SAML elements.
pub fn validate_saml_metadata(metadata: &str) -> Result<()> {
    if metadata.is_empty() {
        return Err(AmiError::InvalidParameter {
            message: "SAML metadata document cannot be empty".to_string(),
        });
    }

    // Parse XML to ensure it's well-formed
    roxmltree::Document::parse(metadata).map_err(|e| AmiError::InvalidParameter {
        message: format!("Invalid SAML metadata XML: {}", e),
    })?;

    // Check for basic SAML structure
    if !metadata.contains("EntityDescriptor") && !metadata.contains("EntitiesDescriptor") {
        return Err(AmiError::InvalidParameter {
            message: "SAML metadata must contain EntityDescriptor or EntitiesDescriptor"
                .to_string(),
        });
    }

    Ok(())
}

/// Extract validity period from SAML metadata
///
/// Attempts to find and parse the validUntil attribute from SAML metadata.
pub fn extract_saml_validity(metadata: &str) -> Result<Option<DateTime<Utc>>> {
    let doc = roxmltree::Document::parse(metadata).map_err(|e| AmiError::InvalidParameter {
        message: format!("Failed to parse SAML metadata: {}", e),
    })?;

    // Look for validUntil attribute
    for node in doc.descendants() {
        if let Some(valid_until) = node.attribute("validUntil") {
            // Try to parse ISO 8601 datetime
            match DateTime::parse_from_rfc3339(valid_until) {
                Ok(dt) => return Ok(Some(dt.with_timezone(&Utc))),
                Err(_) => {
                    // Try alternative formats
                    log::warn!("Could not parse validUntil: {}", valid_until);
                }
            }
        }
    }

    Ok(None)
}

/// Validate OIDC provider URL
///
/// Ensures URL is HTTPS and has proper format.
pub fn validate_oidc_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(AmiError::InvalidParameter {
            message: "OIDC provider URL cannot be empty".to_string(),
        });
    }

    if !url.starts_with("https://") {
        return Err(AmiError::InvalidParameter {
            message: "OIDC provider URL must use HTTPS".to_string(),
        });
    }

    if url.len() > 255 {
        return Err(AmiError::InvalidParameter {
            message: "OIDC provider URL too long (max 255 characters)".to_string(),
        });
    }

    // Basic URL structure validation
    let without_scheme = url.trim_start_matches("https://");
    if without_scheme.is_empty() || !without_scheme.contains('.') {
        return Err(AmiError::InvalidParameter {
            message: "OIDC provider URL must have a valid domain".to_string(),
        });
    }

    Ok(())
}

/// Validate certificate thumbprint format
///
/// Ensures thumbprint is a valid SHA-1 hex string (40 characters).
pub fn validate_thumbprint(thumbprint: &str) -> Result<()> {
    if thumbprint.len() != 40 {
        return Err(AmiError::InvalidParameter {
            message: format!(
                "Thumbprint must be exactly 40 characters (SHA-1 hex), got {}",
                thumbprint.len()
            ),
        });
    }

    if !thumbprint.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AmiError::InvalidParameter {
            message: "Thumbprint must contain only hexadecimal characters (0-9, a-f, A-F)"
                .to_string(),
        });
    }

    Ok(())
}

/// Validate thumbprint list
///
/// Ensures all thumbprints in the list are valid and the list isn't too large.
pub fn validate_thumbprint_list(thumbprints: &[String]) -> Result<()> {
    if thumbprints.is_empty() {
        return Err(AmiError::InvalidParameter {
            message: "At least one thumbprint is required".to_string(),
        });
    }

    if thumbprints.len() > 5 {
        return Err(AmiError::InvalidParameter {
            message: "Maximum 5 thumbprints allowed".to_string(),
        });
    }

    for thumbprint in thumbprints {
        validate_thumbprint(thumbprint)?;
    }

    Ok(())
}

/// Validate client ID list
///
/// Ensures client IDs are valid and the list isn't too large.
pub fn validate_client_id_list(client_ids: &[String]) -> Result<()> {
    if client_ids.is_empty() {
        return Err(AmiError::InvalidParameter {
            message: "At least one client ID is required".to_string(),
        });
    }

    if client_ids.len() > 100 {
        return Err(AmiError::InvalidParameter {
            message: "Maximum 100 client IDs allowed".to_string(),
        });
    }

    for client_id in client_ids {
        if client_id.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Client ID cannot be empty".to_string(),
            });
        }

        if client_id.len() > 255 {
            return Err(AmiError::InvalidParameter {
                message: "Client ID too long (max 255 characters)".to_string(),
            });
        }
    }

    Ok(())
}

/// Parse OIDC discovery document
///
/// Validates basic OIDC discovery document structure.
pub fn parse_oidc_discovery(discovery_json: &str) -> Result<()> {
    let doc: serde_json::Value =
        serde_json::from_str(discovery_json).map_err(|e| AmiError::InvalidParameter {
            message: format!("Invalid OIDC discovery document JSON: {}", e),
        })?;

    // Check for required OIDC discovery fields
    let required_fields = [
        "issuer",
        "authorization_endpoint",
        "token_endpoint",
        "jwks_uri",
    ];

    for field in &required_fields {
        if doc.get(field).is_none() {
            return Err(AmiError::InvalidParameter {
                message: format!("OIDC discovery document missing required field: {}", field),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_saml_metadata() {
        let valid_metadata = r#"<?xml version="1.0"?>
            <EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata">
                <IDPSSODescriptor>
                    <SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" />
                </IDPSSODescriptor>
            </EntityDescriptor>"#;

        assert!(validate_saml_metadata(valid_metadata).is_ok());
        assert!(validate_saml_metadata("").is_err());
        assert!(validate_saml_metadata("not xml").is_err());
        assert!(validate_saml_metadata("<xml>no saml elements</xml>").is_err());
    }

    #[test]
    fn test_extract_saml_validity() {
        let metadata_with_validity = r#"<?xml version="1.0"?>
            <EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata" 
                              validUntil="2025-12-31T23:59:59Z">
                <IDPSSODescriptor />
            </EntityDescriptor>"#;

        let result = extract_saml_validity(metadata_with_validity);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let metadata_without_validity = r#"<?xml version="1.0"?>
            <EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata">
                <IDPSSODescriptor />
            </EntityDescriptor>"#;

        let result2 = extract_saml_validity(metadata_without_validity);
        assert!(result2.is_ok());
        assert!(result2.unwrap().is_none());
    }

    #[test]
    fn test_validate_oidc_url() {
        assert!(validate_oidc_url("https://accounts.google.com").is_ok());
        assert!(validate_oidc_url("https://login.microsoftonline.com/tenant").is_ok());
        assert!(validate_oidc_url("http://example.com").is_err());
        assert!(validate_oidc_url("https://").is_err());
        assert!(validate_oidc_url("").is_err());
        assert!(validate_oidc_url(&format!("https://{}", "a".repeat(260))).is_err());
    }

    #[test]
    fn test_validate_thumbprint() {
        assert!(validate_thumbprint("0123456789abcdef0123456789abcdef01234567").is_ok());
        assert!(validate_thumbprint("ABCDEF0123456789ABCDEF0123456789ABCDEF01").is_ok());
        assert!(validate_thumbprint("short").is_err());
        assert!(validate_thumbprint(&"a".repeat(41)).is_err());
        assert!(validate_thumbprint("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").is_err());
    }

    #[test]
    fn test_validate_thumbprint_list() {
        assert!(validate_thumbprint_list(
            &["0123456789abcdef0123456789abcdef01234567".to_string()]
        )
        .is_ok());

        assert!(validate_thumbprint_list(&[]).is_err());

        assert!(validate_thumbprint_list(&[
            "0123456789abcdef0123456789abcdef01234567".to_string(),
            "invalid".to_string()
        ])
        .is_err());

        assert!(validate_thumbprint_list(&vec!["0".repeat(40); 6]).is_err());
    }

    #[test]
    fn test_validate_client_id_list() {
        assert!(validate_client_id_list(&["client-id-123".to_string()]).is_ok());
        assert!(validate_client_id_list(&[]).is_err());
        assert!(validate_client_id_list(&["".to_string()]).is_err());
        assert!(validate_client_id_list(&["a".repeat(256)]).is_err());
        assert!(validate_client_id_list(&vec!["id".to_string(); 101]).is_err());
    }

    #[test]
    fn test_parse_oidc_discovery() {
        let valid_discovery = r#"{
            "issuer": "https://accounts.google.com",
            "authorization_endpoint": "https://accounts.google.com/o/oauth2/v2/auth",
            "token_endpoint": "https://oauth2.googleapis.com/token",
            "jwks_uri": "https://www.googleapis.com/oauth2/v3/certs"
        }"#;

        assert!(parse_oidc_discovery(valid_discovery).is_ok());
        assert!(parse_oidc_discovery("not json").is_err());
        assert!(parse_oidc_discovery(r#"{"issuer": "test"}"#).is_err());
    }
}
