//! ARN transformers for converting between WAMI ARNs and provider-specific formats.

use super::types::{Service, WamiArn};
use crate::error::{AmiError, Result};

/// Trait for transforming WAMI ARNs to and from provider-specific formats.
pub trait ArnTransformer {
    /// Converts a WAMI ARN to a provider-specific ARN format.
    ///
    /// Returns an error if the ARN is not cloud-synced or if the provider doesn't match.
    #[allow(clippy::result_large_err)]
    fn to_provider_arn(&self, arn: &WamiArn) -> Result<String>;

    /// Attempts to convert a provider-specific ARN back to a WAMI ARN.
    ///
    /// Note: This may require additional context (tenant_path, wami_instance_id)
    /// that may not be present in the provider ARN, so this operation may be lossy.
    #[allow(clippy::wrong_self_convention, clippy::result_large_err)]
    fn from_provider_arn(&self, provider_arn: &str) -> Result<ProviderArnInfo>;
}

/// Information extracted from a provider-specific ARN.
///
/// This doesn't contain the full WAMI context (tenant hierarchy, instance ID)
/// as those are not typically present in provider ARNs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderArnInfo {
    /// The cloud provider name
    pub provider: String,
    /// The provider's account ID
    pub account_id: String,
    /// The service name (may need mapping to WAMI service)
    pub service: String,
    /// Resource type
    pub resource_type: String,
    /// Resource ID
    pub resource_id: String,
    /// Optional region
    pub region: Option<String>,
}

/// AWS ARN transformer.
///
/// Converts between WAMI ARNs and AWS ARN format:
/// `arn:aws:{service}::{account_id}:{resource_type}/{resource_id}`
///
/// # Examples
///
/// ```
/// use wami::arn::{WamiArn, Service, AwsArnTransformer, ArnTransformer};
///
/// let arn = WamiArn::builder()
///     .service(Service::Iam)
///     .tenant("t1")
///     .wami_instance("999888777")
///     .cloud_provider("aws", "223344556677")
///     .resource("user", "77557755")
///     .build()
///     .unwrap();
///
/// let transformer = AwsArnTransformer;
/// let aws_arn = transformer.to_provider_arn(&arn).unwrap();
/// assert_eq!(aws_arn, "arn:aws:iam::223344556677:user/77557755");
/// ```
pub struct AwsArnTransformer;

impl ArnTransformer for AwsArnTransformer {
    fn to_provider_arn(&self, arn: &WamiArn) -> Result<String> {
        let cloud_mapping =
            arn.cloud_mapping
                .as_ref()
                .ok_or_else(|| AmiError::InvalidParameter {
                    message: "ARN is not cloud-synced".to_string(),
                })?;

        if cloud_mapping.provider != "aws" {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "ARN provider is '{}', expected 'aws'",
                    cloud_mapping.provider
                ),
            });
        }

        // Map WAMI service to AWS service
        let aws_service = match &arn.service {
            Service::Iam => "iam",
            Service::Sts => "sts",
            Service::SsoAdmin => "sso",
            Service::Custom(s) => s.as_str(),
        };

        // AWS ARN format: arn:aws:service:region:account-id:resource
        // For global services like IAM, region is empty
        let region = cloud_mapping.region.as_deref().unwrap_or("");

        Ok(format!(
            "arn:aws:{}:{}:{}:{}/{}",
            aws_service,
            region,
            cloud_mapping.account_id,
            arn.resource.resource_type,
            arn.resource.resource_id
        ))
    }

    fn from_provider_arn(&self, provider_arn: &str) -> Result<ProviderArnInfo> {
        let parts: Vec<&str> = provider_arn.split(':').collect();

        if parts.len() < 6 {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid AWS ARN format: expected at least 6 parts, got {}",
                    parts.len()
                ),
            });
        }

        if parts[0] != "arn" || parts[1] != "aws" {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid AWS ARN prefix: expected 'arn:aws', got '{}:{}'",
                    parts[0], parts[1]
                ),
            });
        }

        let service = parts[2].to_string();
        let region = parts[3].to_string(); // Region (may be empty for global services)
        let account_id = parts[4].to_string();

        // Resource is everything after account_id
        let resource_part = parts[5..].join(":");
        let resource_parts: Vec<&str> = resource_part.split('/').collect();

        if resource_parts.len() < 2 {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid AWS ARN resource format: expected 'type/id', got '{}'",
                    resource_part
                ),
            });
        }

        let resource_type = resource_parts[0].to_string();
        let resource_id = resource_parts[1..].join("/");

        Ok(ProviderArnInfo {
            provider: "aws".to_string(),
            account_id,
            service,
            resource_type,
            resource_id,
            region: if region.is_empty() {
                None
            } else {
                Some(region)
            },
        })
    }
}

/// GCP ARN transformer.
///
/// Converts between WAMI ARNs and GCP resource name format:
/// `//iam.googleapis.com/projects/{project_id}/serviceAccounts/{resource_id}`
///
/// Note: GCP uses "resource names" rather than ARNs, but we use a simplified
/// ARN-like format for consistency.
pub struct GcpArnTransformer;

impl ArnTransformer for GcpArnTransformer {
    fn to_provider_arn(&self, arn: &WamiArn) -> Result<String> {
        let cloud_mapping =
            arn.cloud_mapping
                .as_ref()
                .ok_or_else(|| AmiError::InvalidParameter {
                    message: "ARN is not cloud-synced".to_string(),
                })?;

        if cloud_mapping.provider != "gcp" {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "ARN provider is '{}', expected 'gcp'",
                    cloud_mapping.provider
                ),
            });
        }

        // Map WAMI service to GCP service
        let gcp_service = match &arn.service {
            Service::Iam => "iam.googleapis.com",
            Service::SsoAdmin => "cloudidentity.googleapis.com",
            Service::Custom(s) => s.as_str(),
            _ => "iam.googleapis.com", // Default to IAM
        };

        // GCP format - optionally include location if regional
        let base = format!(
            "//{}/projects/{}/{}s/{}",
            gcp_service,
            cloud_mapping.account_id,
            arn.resource.resource_type,
            arn.resource.resource_id
        );

        // If regional, could append location, but GCP format varies by service
        // For simplicity, we keep the basic format
        Ok(base)
    }

    fn from_provider_arn(&self, provider_arn: &str) -> Result<ProviderArnInfo> {
        // Parse GCP resource name format
        // //iam.googleapis.com/projects/{project_id}/{resource_type}s/{resource_id}

        if !provider_arn.starts_with("//") {
            return Err(AmiError::InvalidParameter {
                message: "Invalid GCP resource name: expected '//' prefix".to_string(),
            });
        }

        let parts: Vec<&str> = provider_arn[2..].split('/').collect();

        if parts.len() < 4 {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid GCP resource name format: expected at least 4 parts, got {}",
                    parts.len()
                ),
            });
        }

        let service = parts[0].to_string();

        if parts[1] != "projects" {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid GCP resource name: expected 'projects', got '{}'",
                    parts[1]
                ),
            });
        }

        let account_id = parts[2].to_string();

        // Resource type (remove trailing 's' if present)
        let mut resource_type = parts[3].to_string();
        if resource_type.ends_with('s') {
            resource_type.pop();
        }

        let resource_id = parts[4..].join("/");

        Ok(ProviderArnInfo {
            provider: "gcp".to_string(),
            account_id,
            service,
            resource_type,
            resource_id,
            region: None, // GCP uses locations in a different format
        })
    }
}

/// Azure ARN transformer.
///
/// Converts between WAMI ARNs and Azure resource ID format:
/// `/subscriptions/{subscription_id}/resourceGroups/{resource_group}/providers/Microsoft.{service}/{resource_type}/{resource_id}`
pub struct AzureArnTransformer;

impl ArnTransformer for AzureArnTransformer {
    fn to_provider_arn(&self, arn: &WamiArn) -> Result<String> {
        let cloud_mapping =
            arn.cloud_mapping
                .as_ref()
                .ok_or_else(|| AmiError::InvalidParameter {
                    message: "ARN is not cloud-synced".to_string(),
                })?;

        if cloud_mapping.provider != "azure" {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "ARN provider is '{}', expected 'azure'",
                    cloud_mapping.provider
                ),
            });
        }

        // Map WAMI service to Azure namespace
        let azure_namespace = match &arn.service {
            Service::Iam => "Microsoft.Authorization",
            Service::SsoAdmin => "Microsoft.AzureActiveDirectory",
            Service::Custom(s) => s.as_str(),
            _ => "Microsoft.Authorization",
        };

        // For Azure, account_id is subscription_id
        // We use a default resource group "wami-resources"
        Ok(format!(
            "/subscriptions/{}/resourceGroups/wami-resources/providers/{}/{}/{}",
            cloud_mapping.account_id,
            azure_namespace,
            arn.resource.resource_type,
            arn.resource.resource_id
        ))
    }

    fn from_provider_arn(&self, provider_arn: &str) -> Result<ProviderArnInfo> {
        // Parse Azure resource ID format
        let parts: Vec<&str> = provider_arn.split('/').collect();

        if parts.len() < 9 || !parts[0].is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Invalid Azure resource ID format".to_string(),
            });
        }

        if parts[1] != "subscriptions" {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid Azure resource ID: expected 'subscriptions', got '{}'",
                    parts[1]
                ),
            });
        }

        let account_id = parts[2].to_string();
        let service = parts[5].to_string(); // providers/{namespace}
        let resource_type = parts[6].to_string();
        let resource_id = parts[7..].join("/");

        Ok(ProviderArnInfo {
            provider: "azure".to_string(),
            account_id,
            service,
            resource_type,
            resource_id,
            region: None, // Azure uses locations in resource groups
        })
    }
}

/// Scaleway ARN transformer.
///
/// Converts between WAMI ARNs and Scaleway resource format:
/// `scw:{organization_id}:{service}:{resource_type}/{resource_id}`
pub struct ScalewayArnTransformer;

impl ArnTransformer for ScalewayArnTransformer {
    fn to_provider_arn(&self, arn: &WamiArn) -> Result<String> {
        let cloud_mapping =
            arn.cloud_mapping
                .as_ref()
                .ok_or_else(|| AmiError::InvalidParameter {
                    message: "ARN is not cloud-synced".to_string(),
                })?;

        if cloud_mapping.provider != "scaleway" {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "ARN provider is '{}', expected 'scaleway'",
                    cloud_mapping.provider
                ),
            });
        }

        let service = match &arn.service {
            Service::Iam => "iam",
            Service::SsoAdmin => "sso",
            Service::Custom(s) => s.as_str(),
            _ => "iam",
        };

        Ok(format!(
            "scw:{}:{}:{}/{}",
            cloud_mapping.account_id, service, arn.resource.resource_type, arn.resource.resource_id
        ))
    }

    fn from_provider_arn(&self, provider_arn: &str) -> Result<ProviderArnInfo> {
        let parts: Vec<&str> = provider_arn.split(':').collect();

        if parts.len() < 4 {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid Scaleway resource format: expected at least 4 parts, got {}",
                    parts.len()
                ),
            });
        }

        if parts[0] != "scw" {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid Scaleway resource prefix: expected 'scw', got '{}'",
                    parts[0]
                ),
            });
        }

        let account_id = parts[1].to_string();
        let service = parts[2].to_string();

        let resource_part = parts[3..].join(":");
        let resource_parts: Vec<&str> = resource_part.split('/').collect();

        if resource_parts.len() < 2 {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid Scaleway resource format: expected 'type/id', got '{}'",
                    resource_part
                ),
            });
        }

        let resource_type = resource_parts[0].to_string();
        let resource_id = resource_parts[1..].join("/");

        Ok(ProviderArnInfo {
            provider: "scaleway".to_string(),
            account_id,
            service,
            resource_type,
            resource_id,
            region: None, // Scaleway regions handled separately
        })
    }
}

/// Gets the appropriate transformer for a given provider.
///
/// # Examples
///
/// ```
/// use wami::arn::get_transformer;
///
/// let transformer = get_transformer("aws");
/// assert!(transformer.is_some());
///
/// let transformer = get_transformer("unknown");
/// assert!(transformer.is_none());
/// ```
pub fn get_transformer(provider: &str) -> Option<Box<dyn ArnTransformer>> {
    match provider {
        "aws" => Some(Box::new(AwsArnTransformer)),
        "gcp" => Some(Box::new(GcpArnTransformer)),
        "azure" => Some(Box::new(AzureArnTransformer)),
        "scaleway" => Some(Box::new(ScalewayArnTransformer)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{Service, WamiArn};

    #[test]
    fn test_aws_transformer() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider("aws", "223344556677")
            .resource("user", "77557755")
            .build()
            .unwrap();

        let transformer = AwsArnTransformer;
        let aws_arn = transformer.to_provider_arn(&arn).unwrap();
        assert_eq!(aws_arn, "arn:aws:iam::223344556677:user/77557755");

        let info = transformer.from_provider_arn(&aws_arn).unwrap();
        assert_eq!(info.provider, "aws");
        assert_eq!(info.account_id, "223344556677");
        assert_eq!(info.service, "iam");
        assert_eq!(info.resource_type, "user");
        assert_eq!(info.resource_id, "77557755");
    }

    #[test]
    fn test_aws_transformer_wrong_provider() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider("gcp", "123456")
            .resource("user", "77557755")
            .build()
            .unwrap();

        let transformer = AwsArnTransformer;
        let result = transformer.to_provider_arn(&arn);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected 'aws'"));
    }

    #[test]
    fn test_aws_transformer_not_cloud_synced() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .resource("user", "77557755")
            .build()
            .unwrap();

        let transformer = AwsArnTransformer;
        let result = transformer.to_provider_arn(&arn);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not cloud-synced"));
    }

    #[test]
    fn test_gcp_transformer() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider("gcp", "554433221")
            .resource("serviceAccount", "77557755")
            .build()
            .unwrap();

        let transformer = GcpArnTransformer;
        let gcp_arn = transformer.to_provider_arn(&arn).unwrap();
        assert_eq!(
            gcp_arn,
            "//iam.googleapis.com/projects/554433221/serviceAccounts/77557755"
        );
    }

    #[test]
    fn test_azure_transformer() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider("azure", "sub-12345")
            .resource("user", "77557755")
            .build()
            .unwrap();

        let transformer = AzureArnTransformer;
        let azure_arn = transformer.to_provider_arn(&arn).unwrap();
        assert_eq!(
            azure_arn,
            "/subscriptions/sub-12345/resourceGroups/wami-resources/providers/Microsoft.Authorization/user/77557755"
        );
    }

    #[test]
    fn test_scaleway_transformer() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider("scaleway", "112233445")
            .resource("user", "77557755")
            .build()
            .unwrap();

        let transformer = ScalewayArnTransformer;
        let scw_arn = transformer.to_provider_arn(&arn).unwrap();
        assert_eq!(scw_arn, "scw:112233445:iam:user/77557755");

        let info = transformer.from_provider_arn(&scw_arn).unwrap();
        assert_eq!(info.provider, "scaleway");
        assert_eq!(info.account_id, "112233445");
        assert_eq!(info.service, "iam");
        assert_eq!(info.resource_type, "user");
        assert_eq!(info.resource_id, "77557755");
    }

    #[test]
    fn test_get_transformer() {
        assert!(get_transformer("aws").is_some());
        assert!(get_transformer("gcp").is_some());
        assert!(get_transformer("azure").is_some());
        assert!(get_transformer("scaleway").is_some());
        assert!(get_transformer("unknown").is_none());
    }

    #[test]
    fn test_aws_sts_service() {
        let arn = WamiArn::builder()
            .service(Service::Sts)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider("aws", "223344556677")
            .resource("assumed-role", "role123")
            .build()
            .unwrap();

        let transformer = AwsArnTransformer;
        let aws_arn = transformer.to_provider_arn(&arn).unwrap();
        assert_eq!(aws_arn, "arn:aws:sts::223344556677:assumed-role/role123");
    }

    #[test]
    fn test_aws_resource_with_slash() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider("aws", "223344556677")
            .resource("policy", "path/to/policy")
            .build()
            .unwrap();

        let transformer = AwsArnTransformer;
        let aws_arn = transformer.to_provider_arn(&arn).unwrap();
        assert_eq!(aws_arn, "arn:aws:iam::223344556677:policy/path/to/policy");

        let info = transformer.from_provider_arn(&aws_arn).unwrap();
        assert_eq!(info.resource_id, "path/to/policy");
    }

    #[test]
    fn test_aws_transformer_with_region() {
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider_with_region("aws", "223344556677", "us-east-1")
            .resource("user", "alice")
            .build()
            .unwrap();

        let transformer = AwsArnTransformer;
        let aws_arn = transformer.to_provider_arn(&arn).unwrap();
        // The implementation includes region in the ARN format
        assert_eq!(aws_arn, "arn:aws:iam:us-east-1:223344556677:user/alice");
    }

    #[test]
    fn test_aws_transformer_invalid_arn_format() {
        let transformer = AwsArnTransformer;
        let result = transformer.from_provider_arn("not-an-arn");
        assert!(result.is_err());
    }

    #[test]
    fn test_gcp_transformer_error_cases() {
        let transformer = GcpArnTransformer;

        // Not cloud synced
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .resource("user", "alice")
            .build()
            .unwrap();

        let result = transformer.to_provider_arn(&arn);
        assert!(result.is_err());

        // Wrong provider
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .cloud_provider("aws", "123456")
            .resource("user", "alice")
            .build()
            .unwrap();

        let result = transformer.to_provider_arn(&arn);
        assert!(result.is_err());
    }

    #[test]
    fn test_azure_transformer_error_cases() {
        let transformer = AzureArnTransformer;

        // Invalid Azure resource ID format
        let result = transformer.from_provider_arn("/invalid");
        assert!(result.is_err());

        // Missing required parts
        let result = transformer.from_provider_arn("/subscriptions/123");
        assert!(result.is_err());
    }

    #[test]
    fn test_scaleway_transformer_error_cases() {
        let transformer = ScalewayArnTransformer;

        // Invalid format
        let result = transformer.from_provider_arn("invalid");
        assert!(result.is_err());

        // Not cloud synced
        let arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678)
            .wami_instance("999888777")
            .resource("user", "alice")
            .build()
            .unwrap();

        let result = transformer.to_provider_arn(&arn);
        assert!(result.is_err());
    }

    #[test]
    fn test_aws_transformer_from_arn_edge_cases() {
        let transformer = AwsArnTransformer;

        // ARN with assumed role
        let aws_arn = "arn:aws:sts::123456789012:assumed-role/MyRole/session";
        let info = transformer.from_provider_arn(aws_arn).unwrap();
        assert_eq!(info.provider, "aws");
        assert_eq!(info.account_id, "123456789012");
        assert_eq!(info.service, "sts");
        assert_eq!(info.resource_type, "assumed-role");
        assert_eq!(info.resource_id, "MyRole/session");
        assert_eq!(info.region, None);

        // ARN with region (for services that support it)
        let aws_arn = "arn:aws:s3:us-east-1::bucket/my-bucket";
        let info = transformer.from_provider_arn(aws_arn).unwrap();
        assert_eq!(info.service, "s3");
        assert_eq!(info.region, Some("us-east-1".to_string()));
    }
}
