//! Provider Information for Multi-Cloud Support
//!
//! Tracks native cloud provider details for WAMI resources.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about a cloud provider where a resource exists
///
/// # Purpose
///
/// While WAMI uses opaque ARNs externally (`arn:wami:iam:tenant-xxx:user/alice`),
/// we need to track the real native cloud identifiers for interoperability.
///
/// # Example
///
/// ```rust
/// use wami::provider::provider_info::ProviderInfo;
/// use chrono::Utc;
///
/// let aws_info = ProviderInfo {
///     provider_type: "aws".to_string(),
///     native_arn: "arn:aws:iam::123456789012:user/alice".to_string(),
///     resource_id: Some("AIDACKCEVSQ6C2EXAMPLE".to_string()),
///     account_id: "123456789012".to_string(),
///     synced_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderInfo {
    /// Provider type ("aws", "gcp", "azure", "custom")
    pub provider_type: String,

    /// Native ARN/URN/Resource ID from the cloud provider
    ///
    /// Examples:
    /// - AWS: `arn:aws:iam::123456789012:user/alice`
    /// - GCP: `projects/my-project/serviceAccounts/alice@...`
    /// - Azure: `/subscriptions/.../providers/Microsoft.Authorization/users/alice`
    pub native_arn: String,

    /// Provider-specific resource ID (optional)
    ///
    /// Examples:
    /// - AWS: `AIDACKCEVSQ6C2EXAMPLE`
    /// - GCP: `123456789012345678`
    /// - Azure: `550e8400-e29b-41d4-a716-446655440000`
    pub resource_id: Option<String>,

    /// Real account/project/subscription ID (non-obfuscated)
    pub account_id: String,

    /// When this provider entry was created/synced
    pub synced_at: DateTime<Utc>,
}

impl ProviderInfo {
    /// Creates a new ProviderInfo with current timestamp
    pub fn new(
        provider_type: impl Into<String>,
        native_arn: impl Into<String>,
        resource_id: Option<String>,
        account_id: impl Into<String>,
    ) -> Self {
        Self {
            provider_type: provider_type.into(),
            native_arn: native_arn.into(),
            resource_id,
            account_id: account_id.into(),
            synced_at: Utc::now(),
        }
    }

    /// Checks if this provider is AWS
    pub fn is_aws(&self) -> bool {
        self.provider_type == "aws"
    }

    /// Checks if this provider is GCP
    pub fn is_gcp(&self) -> bool {
        self.provider_type == "gcp"
    }

    /// Checks if this provider is Azure
    pub fn is_azure(&self) -> bool {
        self.provider_type == "azure"
    }

    /// Checks if this is a custom provider
    pub fn is_custom(&self) -> bool {
        !self.is_aws() && !self.is_gcp() && !self.is_azure()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info_creation() {
        let info = ProviderInfo::new(
            "aws",
            "arn:aws:iam::123456789012:user/alice",
            Some("AIDACKCEVSQ6C2EXAMPLE".to_string()),
            "123456789012",
        );

        assert_eq!(info.provider_type, "aws");
        assert!(info.is_aws());
        assert!(!info.is_gcp());
    }

    #[test]
    fn test_provider_type_checks() {
        let aws = ProviderInfo::new("aws", "arn:...", None, "123");
        let gcp = ProviderInfo::new("gcp", "projects/...", None, "project-id");
        let azure = ProviderInfo::new("azure", "/subscriptions/...", None, "sub-id");
        let custom = ProviderInfo::new("mycloud", "mycloud://...", None, "tenant-123");

        assert!(aws.is_aws());
        assert!(gcp.is_gcp());
        assert!(azure.is_azure());
        assert!(custom.is_custom());
    }
}
