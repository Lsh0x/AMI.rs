//! ARN parsing and serialization.

use super::types::{CloudMapping, Resource, Service, TenantPath, WamiArn};
use crate::error::{AmiError, Result};
use std::str::FromStr;

/// Error type for ARN parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArnParseError {
    /// Invalid ARN format
    InvalidFormat(String),
    /// Missing required component
    MissingComponent(String),
    /// Invalid component value
    InvalidComponent(String),
}

impl std::fmt::Display for ArnParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArnParseError::InvalidFormat(msg) => write!(f, "Invalid ARN format: {}", msg),
            ArnParseError::MissingComponent(msg) => write!(f, "Missing ARN component: {}", msg),
            ArnParseError::InvalidComponent(msg) => write!(f, "Invalid ARN component: {}", msg),
        }
    }
}

impl std::error::Error for ArnParseError {}

impl From<ArnParseError> for AmiError {
    fn from(err: ArnParseError) -> Self {
        AmiError::InvalidParameter {
            message: err.to_string(),
        }
    }
}

impl FromStr for WamiArn {
    type Err = ArnParseError;

    /// Parses a WAMI ARN from a string.
    ///
    /// # Format
    ///
    /// ## WAMI Native:
    /// ```text
    /// arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{resource_type}/{resource_id}
    /// ```
    ///
    /// ## Cloud-Synced:
    /// ```text
    /// arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{provider}:{account_id}:{resource_type}/{resource_id}
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use wami::arn::WamiArn;
    /// use std::str::FromStr;
    ///
    /// let arn = WamiArn::from_str("arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/77557755").unwrap();
    /// assert_eq!(arn.resource_type(), "user");
    /// assert_eq!(arn.resource_id(), "77557755");
    ///
    /// let arn = WamiArn::from_str("arn:wami:iam:12345678:wami:999888777:aws:223344556677:user/77557755").unwrap();
    /// assert!(arn.is_cloud_synced());
    /// assert_eq!(arn.provider(), Some("aws"));
    /// ```
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // Split by ':' first
        let parts: Vec<&str> = s.split(':').collect();

        // Minimum parts: arn:wami:service:tenant:wami:instance:resource
        if parts.len() < 7 {
            return Err(ArnParseError::InvalidFormat(format!(
                "Expected at least 7 parts, got {}",
                parts.len()
            )));
        }

        // Validate prefix
        if parts[0] != "arn" {
            return Err(ArnParseError::InvalidFormat(format!(
                "Expected 'arn' prefix, got '{}'",
                parts[0]
            )));
        }

        if parts[1] != "wami" {
            return Err(ArnParseError::InvalidFormat(format!(
                "Expected 'wami' namespace, got '{}'",
                parts[1]
            )));
        }

        // Parse service
        let service = Service::from(parts[2]);

        // Parse tenant path (numeric segments separated by '/')
        let tenant_segments: std::result::Result<Vec<u64>, ArnParseError> = parts[3]
            .split('/')
            .map(|s| {
                s.parse::<u64>().map_err(|_| {
                    ArnParseError::InvalidComponent(format!(
                        "Invalid tenant path segment: '{}' (must be a u64)",
                        s
                    ))
                })
            })
            .collect();

        let tenant_segments = tenant_segments?;

        if tenant_segments.is_empty() {
            return Err(ArnParseError::InvalidComponent(
                "Tenant path cannot be empty".to_string(),
            ));
        }

        let tenant_path = TenantPath::new(tenant_segments);

        // Validate "wami" marker
        if parts[4] != "wami" {
            return Err(ArnParseError::InvalidFormat(format!(
                "Expected 'wami' marker at position 4, got '{}'",
                parts[4]
            )));
        }

        // Parse WAMI instance ID
        let wami_instance_id = parts[5].to_string();
        if wami_instance_id.is_empty() {
            return Err(ArnParseError::InvalidComponent(
                "WAMI instance ID cannot be empty".to_string(),
            ));
        }

        // Now we need to determine if this is a cloud-synced ARN
        // Cloud-synced: arn:wami:service:tenant:wami:instance:provider:account:region:resource
        // Native:       arn:wami:service:tenant:wami:instance:resource

        let (cloud_mapping, resource_part) = if parts.len() >= 10 {
            // Potentially cloud-synced with region (provider:account:region:resource)
            // Check if parts[6], [7], [8] look like provider/account/region (not containing '/')
            if !parts[6].contains('/') && !parts[7].contains('/') && !parts[8].contains('/') {
                // Cloud-synced format with region
                let provider = parts[6].to_string();
                let account_id = parts[7].to_string();
                let region = parts[8].to_string();

                if provider.is_empty() || account_id.is_empty() || region.is_empty() {
                    return Err(ArnParseError::InvalidComponent(
                        "Provider, account ID, and region cannot be empty".to_string(),
                    ));
                }

                let cloud_mapping = if region == "global" {
                    Some(CloudMapping::new(provider, account_id))
                } else {
                    Some(CloudMapping::with_region(provider, account_id, region))
                };

                // Resource is everything after region, joined back with ':'
                let resource_part = parts[9..].join(":");

                (cloud_mapping, resource_part)
            } else {
                // Native format (resource contains ':')
                let resource_part = parts[6..].join(":");
                (None, resource_part)
            }
        } else if parts.len() >= 9 {
            // Legacy cloud-synced without region (provider:account:resource)
            // Check if parts[6] and [7] look like provider/account (not containing '/')
            if !parts[6].contains('/') && !parts[7].contains('/') {
                // Legacy cloud-synced format without region
                let provider = parts[6].to_string();
                let account_id = parts[7].to_string();

                if provider.is_empty() || account_id.is_empty() {
                    return Err(ArnParseError::InvalidComponent(
                        "Provider and account ID cannot be empty".to_string(),
                    ));
                }

                let cloud_mapping = Some(CloudMapping::new(provider, account_id));

                // Resource is everything after account_id, joined back with ':'
                let resource_part = parts[8..].join(":");

                (cloud_mapping, resource_part)
            } else {
                // Native format (resource contains ':')
                let resource_part = parts[6..].join(":");
                (None, resource_part)
            }
        } else {
            // Native format
            let resource_part = parts[6..].join(":");
            (None, resource_part)
        };

        // Parse resource (type/id)
        let resource_parts: Vec<&str> = resource_part.split('/').collect();
        if resource_parts.len() < 2 {
            return Err(ArnParseError::InvalidFormat(format!(
                "Resource must be in format 'type/id', got '{}'",
                resource_part
            )));
        }

        let resource_type = resource_parts[0].to_string();
        let resource_id = resource_parts[1..].join("/"); // Handle resource IDs with '/'

        if resource_type.is_empty() || resource_id.is_empty() {
            return Err(ArnParseError::InvalidComponent(
                "Resource type and ID cannot be empty".to_string(),
            ));
        }

        let resource = Resource::new(resource_type, resource_id);

        Ok(WamiArn {
            service,
            tenant_path,
            wami_instance_id,
            cloud_mapping,
            resource,
        })
    }
}

/// Parses a WAMI ARN from a string, returning a Result with AmiError.
///
/// This is a convenience function that wraps FromStr and converts the error.
///
/// # Examples
///
/// ```
/// use wami::arn::parse_arn;
///
/// let arn = parse_arn("arn:wami:iam:12345678:wami:999888777:user/77557755").unwrap();
/// assert_eq!(arn.resource_type(), "user");
/// ```
#[allow(clippy::result_large_err)]
pub fn parse_arn(s: &str) -> Result<WamiArn> {
    WamiArn::from_str(s).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wami_native() {
        let arn_str = "arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/77557755";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.service, Service::Iam);
        assert_eq!(arn.tenant_path.segments, vec![12345678, 87654321, 99999999]);
        assert_eq!(arn.wami_instance_id, "999888777");
        assert_eq!(arn.cloud_mapping, None);
        assert_eq!(arn.resource.resource_type, "user");
        assert_eq!(arn.resource.resource_id, "77557755");
        assert!(!arn.is_cloud_synced());
    }

    #[test]
    fn test_parse_cloud_synced_aws() {
        let arn_str = "arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:global:user/77557755";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.service, Service::Iam);
        assert_eq!(arn.tenant_path.segments, vec![12345678, 87654321, 99999999]);
        assert_eq!(arn.wami_instance_id, "999888777");
        assert!(arn.is_cloud_synced());
        assert_eq!(arn.cloud_mapping.as_ref().unwrap().provider, "aws");
        assert_eq!(
            arn.cloud_mapping.as_ref().unwrap().account_id,
            "223344556677"
        );
        assert_eq!(arn.cloud_mapping.as_ref().unwrap().region, None);
        assert_eq!(arn.resource.resource_type, "user");
        assert_eq!(arn.resource.resource_id, "77557755");
    }

    #[test]
    fn test_parse_cloud_synced_with_region() {
        let arn_str =
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:us-east-1:user/77557755";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.service, Service::Iam);
        assert!(arn.is_cloud_synced());
        assert_eq!(
            arn.cloud_mapping.as_ref().unwrap().region,
            Some("us-east-1".to_string())
        );
        assert!(arn.cloud_mapping.as_ref().unwrap().is_regional());
    }

    #[test]
    fn test_parse_legacy_cloud_synced_without_region() {
        // Support legacy format without region for backward compatibility
        let arn_str = "arn:wami:iam:12345678:wami:999888777:aws:223344556677:user/77557755";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.service, Service::Iam);
        assert!(arn.is_cloud_synced());
        assert_eq!(arn.cloud_mapping.as_ref().unwrap().region, None);
    }

    #[test]
    fn test_parse_cloud_synced_gcp() {
        let arn_str =
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:gcp:554433221:us-central1:user/77557755";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.cloud_mapping.as_ref().unwrap().provider, "gcp");
        assert_eq!(arn.cloud_mapping.as_ref().unwrap().account_id, "554433221");
        assert_eq!(
            arn.cloud_mapping.as_ref().unwrap().region,
            Some("us-central1".to_string())
        );
    }

    #[test]
    fn test_parse_cloud_synced_scaleway() {
        let arn_str =
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:scaleway:112233445:fr-par:user/77557755";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.cloud_mapping.as_ref().unwrap().provider, "scaleway");
        assert_eq!(arn.cloud_mapping.as_ref().unwrap().account_id, "112233445");
        assert_eq!(
            arn.cloud_mapping.as_ref().unwrap().region,
            Some("fr-par".to_string())
        );
    }

    #[test]
    fn test_parse_single_tenant() {
        let arn_str = "arn:wami:sts:12345678:wami:111222333:session/sess123";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.service, Service::Sts);
        assert_eq!(arn.tenant_path.segments, vec![12345678]);
        assert_eq!(arn.wami_instance_id, "111222333");
        assert_eq!(arn.resource.resource_type, "session");
        assert_eq!(arn.resource.resource_id, "sess123");
    }

    #[test]
    fn test_parse_custom_service() {
        let arn_str = "arn:wami:iam:12345678:wami:999888777:resource/res123";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.service, Service::Custom("custom-service".to_string()));
    }

    #[test]
    fn test_parse_sso_admin() {
        let arn_str = "arn:wami:iam:12345678:wami:999888777:permission-set/ps123";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.service, Service::SsoAdmin);
        assert_eq!(arn.resource.resource_type, "permission-set");
        assert_eq!(arn.resource.resource_id, "ps123");
    }

    #[test]
    fn test_parse_resource_with_slash() {
        let arn_str = "arn:wami:iam:12345678:wami:999888777:policy/path/to/policy";
        let arn = WamiArn::from_str(arn_str).unwrap();

        assert_eq!(arn.resource.resource_type, "policy");
        assert_eq!(arn.resource.resource_id, "path/to/policy");
    }

    #[test]
    fn test_parse_roundtrip_native() {
        let original = "arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/77557755";
        let arn = WamiArn::from_str(original).unwrap();
        let serialized = arn.to_string();
        assert_eq!(original, serialized);
    }

    #[test]
    fn test_parse_roundtrip_cloud_synced_global() {
        let original = "arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:global:user/77557755";
        let arn = WamiArn::from_str(original).unwrap();
        let serialized = arn.to_string();
        assert_eq!(original, serialized);
    }

    #[test]
    fn test_parse_roundtrip_cloud_synced_regional() {
        let original =
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:us-east-1:user/77557755";
        let arn = WamiArn::from_str(original).unwrap();
        let serialized = arn.to_string();
        assert_eq!(original, serialized);
    }

    #[test]
    fn test_parse_invalid_prefix() {
        let result = WamiArn::from_str("invalid:wami:iam:t1:wami:999888777:user/77557755");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected 'arn' prefix"));
    }

    #[test]
    fn test_parse_invalid_namespace() {
        let result = WamiArn::from_str("arn:invalid:iam:t1:wami:999888777:user/77557755");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected 'wami' namespace"));
    }

    #[test]
    fn test_parse_missing_wami_marker() {
        let result = WamiArn::from_str("arn:wami:iam:12345678:invalid:999888777:user/77557755");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected 'wami' marker"));
    }

    #[test]
    fn test_parse_empty_tenant() {
        let result = WamiArn::from_str("arn:wami:iam::wami:999888777:user/77557755");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Tenant path cannot be empty"));
    }

    #[test]
    fn test_parse_empty_instance_id() {
        let result = WamiArn::from_str("arn:wami:iam:12345678:wami::user/77557755");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("WAMI instance ID cannot be empty"));
    }

    #[test]
    fn test_parse_invalid_resource_format() {
        let result = WamiArn::from_str("arn:wami:iam:12345678:wami:999888777:invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Resource must be in format 'type/id'"));
    }

    #[test]
    fn test_parse_too_few_parts() {
        let result = WamiArn::from_str("arn:wami:iam:12345678");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected at least 7 parts"));
    }

    #[test]
    fn test_parse_arn_function() {
        let result = parse_arn("arn:wami:iam:12345678:wami:999888777:user/77557755");
        assert!(result.is_ok());

        let result = parse_arn("invalid");
        assert!(result.is_err());
    }
}
