//! Identity Provider Domain Models
//!
//! Represents SAML and OIDC identity providers for federated authentication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a SAML 2.0 identity provider
///
/// SAML providers enable federated authentication with enterprise identity providers
/// like Okta, Azure AD, or any SAML 2.0 compliant system.
///
/// # Example
///
/// ```rust
/// use wami::wami::identity::identity_provider::SamlProvider;
/// use chrono::Utc;
///
/// let provider = SamlProvider {
///     arn: "arn:aws:iam::123456789012:saml-provider/ExampleProvider".to_string(),
///     saml_provider_name: "ExampleProvider".to_string(),
///     saml_metadata_document: "<EntityDescriptor ...>...</EntityDescriptor>".to_string(),
///     create_date: Utc::now(),
///     valid_until: None,
///     tags: vec![],
///     wami_arn: "arn:wami:iam::tenant-abc:saml-provider/ExampleProvider".to_string(),
///     providers: vec![],
///     tenant_id: None,
///     usage_count: 0,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlProvider {
    /// The ARN (Amazon Resource Name) that identifies the SAML provider
    pub arn: String,
    /// The name of the SAML provider
    pub saml_provider_name: String,
    /// The SAML metadata document (XML format)
    pub saml_metadata_document: String,
    /// The date and time when the provider was created
    pub create_date: DateTime<Utc>,
    /// The date and time when the provider certificate expires
    pub valid_until: Option<DateTime<Utc>>,
    /// A list of tags associated with the provider
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
    /// Optional tenant ID for multi-tenant isolation
    pub tenant_id: Option<crate::wami::tenant::TenantId>,
    /// Number of principals using this provider (for audit/tracking)
    pub usage_count: u64,
}

/// Represents an OpenID Connect (OIDC) identity provider
///
/// OIDC providers enable federated authentication with modern identity providers
/// like Google, Auth0, Cognito, or any OIDC-compliant system.
///
/// # Example
///
/// ```rust
/// use wami::wami::identity::identity_provider::OidcProvider;
/// use chrono::Utc;
///
/// let provider = OidcProvider {
///     arn: "arn:aws:iam::123456789012:oidc-provider/accounts.google.com".to_string(),
///     url: "https://accounts.google.com".to_string(),
///     client_id_list: vec!["my-app-123.apps.googleusercontent.com".to_string()],
///     thumbprint_list: vec!["0123456789abcdef0123456789abcdef01234567".to_string()],
///     create_date: Utc::now(),
///     tags: vec![],
///     wami_arn: "arn:wami:iam::tenant-abc:oidc-provider/accounts.google.com".to_string(),
///     providers: vec![],
///     tenant_id: None,
///     usage_count: 0,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProvider {
    /// The ARN (Amazon Resource Name) that identifies the OIDC provider
    pub arn: String,
    /// The URL of the OIDC provider (e.g., "https://accounts.google.com")
    pub url: String,
    /// List of client IDs (audience) that are allowed to use this provider
    pub client_id_list: Vec<String>,
    /// List of server certificate thumbprints (SHA-1 fingerprints)
    pub thumbprint_list: Vec<String>,
    /// The date and time when the provider was created
    pub create_date: DateTime<Utc>,
    /// A list of tags associated with the provider
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
    /// Optional tenant ID for multi-tenant isolation
    pub tenant_id: Option<crate::wami::tenant::TenantId>,
    /// Number of principals using this provider (for audit/tracking)
    pub usage_count: u64,
}

impl SamlProvider {
    /// Validate SAML provider name format
    #[allow(clippy::result_large_err)]
    pub fn validate_name(name: &str) -> crate::error::Result<()> {
        if name.is_empty() || name.len() > 128 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "SAML provider name must be 1-128 characters".to_string(),
            });
        }
        // Name can contain alphanumeric, +=,.@-_
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || "+=,.@-_".contains(c))
        {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "SAML provider name contains invalid characters".to_string(),
            });
        }
        Ok(())
    }
}

impl OidcProvider {
    /// Validate OIDC provider URL format
    #[allow(clippy::result_large_err)]
    pub fn validate_url(url: &str) -> crate::error::Result<()> {
        if !url.starts_with("https://") {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "OIDC provider URL must use HTTPS".to_string(),
            });
        }
        if url.len() > 255 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "OIDC provider URL too long (max 255 characters)".to_string(),
            });
        }
        Ok(())
    }

    /// Validate thumbprint format (40-character hex SHA-1)
    #[allow(clippy::result_large_err)]
    pub fn validate_thumbprint(thumbprint: &str) -> crate::error::Result<()> {
        if thumbprint.len() != 40 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Thumbprint must be exactly 40 characters (SHA-1 hex)".to_string(),
            });
        }
        if !thumbprint.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Thumbprint must contain only hexadecimal characters".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saml_provider_validate_name() {
        assert!(SamlProvider::validate_name("ValidName").is_ok());
        assert!(SamlProvider::validate_name("Valid-Name_123").is_ok());
        assert!(SamlProvider::validate_name("name@domain.com").is_ok());
        assert!(SamlProvider::validate_name("").is_err());
        assert!(SamlProvider::validate_name(&"a".repeat(129)).is_err());
        assert!(SamlProvider::validate_name("invalid name with spaces").is_err());
    }

    #[test]
    fn test_oidc_provider_validate_url() {
        assert!(OidcProvider::validate_url("https://accounts.google.com").is_ok());
        assert!(OidcProvider::validate_url("https://login.microsoftonline.com").is_ok());
        assert!(OidcProvider::validate_url("http://example.com").is_err());
        assert!(OidcProvider::validate_url(&format!("https://{}", "a".repeat(260))).is_err());
    }

    #[test]
    fn test_oidc_provider_validate_thumbprint() {
        assert!(
            OidcProvider::validate_thumbprint("0123456789abcdef0123456789abcdef01234567").is_ok()
        );
        assert!(
            OidcProvider::validate_thumbprint("ABCDEF0123456789ABCDEF0123456789ABCDEF01").is_ok()
        );
        assert!(OidcProvider::validate_thumbprint("short").is_err());
        assert!(OidcProvider::validate_thumbprint(&"a".repeat(41)).is_err());
        assert!(
            OidcProvider::validate_thumbprint("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").is_err()
        );
    }
}
