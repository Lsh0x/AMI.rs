//! AWS Cloud Provider Implementation
//!
//! This module contains the AWS-specific implementation of the CloudProvider trait,
//! including AWS ARN formats, ID generation patterns, and resource limits.

use super::{CloudProvider, ResourceLimits, ResourceType};
use crate::error::{AmiError, Result};

/// AWS cloud provider implementation
///
/// Implements AWS-specific logic for:
/// - ARN format: `arn:aws:iam::account:resource/path/name`
/// - ID prefixes: AIDA (users), AGPA (groups), AROA (roles), etc.
/// - Resource limits: 2 access keys, 50 tags, etc.
/// - Validation rules: service names, paths, session durations
#[derive(Debug, Clone)]
pub struct AwsProvider {
    limits: ResourceLimits,
}

impl Default for AwsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AwsProvider {
    /// Creates a new AWS provider with default AWS limits
    pub fn new() -> Self {
        Self {
            limits: ResourceLimits::default(),
        }
    }

    /// Creates an AWS provider with custom resource limits
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::provider::{AwsProvider, ResourceLimits};
    ///
    /// let limits = ResourceLimits {
    ///     max_access_keys_per_user: 5, // Custom limit
    ///     ..Default::default()
    /// };
    /// let provider = AwsProvider::with_limits(limits);
    /// ```
    pub fn with_limits(limits: ResourceLimits) -> Self {
        Self { limits }
    }

    /// Extracts service name from AWS service principal
    ///
    /// # Example
    ///
    /// ```
    /// # use wami::provider::aws::AwsProvider;
    /// let service = AwsProvider::extract_service_name("elasticbeanstalk.amazonaws.com");
    /// assert_eq!(service, Some("elasticbeanstalk"));
    /// ```
    pub fn extract_service_name(service_principal: &str) -> Option<&str> {
        service_principal.split('.').next()
    }

    /// Converts service name to PascalCase
    ///
    /// # Example
    ///
    /// ```
    /// # use wami::provider::aws::AwsProvider;
    /// let pascal = AwsProvider::to_pascal_case("elastic-beanstalk");
    /// assert_eq!(pascal, "ElasticBeanstalk");
    /// ```
    pub fn to_pascal_case(name: &str) -> String {
        name.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }

    /// Generates a random alphanumeric string of specified length
    fn random_alphanumeric(length: usize) -> String {
        uuid::Uuid::new_v4()
            .to_string()
            .replace('-', "")
            .chars()
            .take(length)
            .collect()
    }
}

impl CloudProvider for AwsProvider {
    fn name(&self) -> &str {
        "aws"
    }

    fn generate_resource_identifier(
        &self,
        resource_type: ResourceType,
        account_id: &str,
        path: &str,
        name: &str,
    ) -> String {
        let resource_name = match resource_type {
            ResourceType::User => "user",
            ResourceType::Group => "group",
            ResourceType::Role => "role",
            ResourceType::Policy => "policy",
            ResourceType::MfaDevice => "mfa",
            ResourceType::AccessKey => "access-key",
            ResourceType::ServerCertificate => "server-certificate",
            ResourceType::ServiceCredential => "service-credential",
            ResourceType::SigningCertificate => "signing-certificate",
        };

        // AWS ARN format: arn:aws:iam::account_id:resource_type/path/name
        format!(
            "arn:aws:iam::{}:{}{}{}",
            account_id, resource_name, path, name
        )
    }

    fn generate_resource_id(&self, resource_type: ResourceType) -> String {
        // AWS uses specific 4-letter prefixes for different resource types
        let prefix = match resource_type {
            ResourceType::User => "AIDA",
            ResourceType::Group => "AGPA",
            ResourceType::Role => "AROA",
            ResourceType::Policy => "ANPA",
            ResourceType::AccessKey => "AKIA",
            ResourceType::ServerCertificate => "ASCA",
            ResourceType::ServiceCredential => "ACCA",
            ResourceType::MfaDevice => "AMFA",
            ResourceType::SigningCertificate => "ASCA",
        };

        // AWS IDs are: 4-letter prefix + 17 random alphanumeric characters
        format!("{}{}", prefix, Self::random_alphanumeric(17))
    }

    fn resource_limits(&self) -> &ResourceLimits {
        &self.limits
    }

    fn validate_service_name(&self, service: &str) -> Result<()> {
        // AWS service names must end with .amazonaws.com
        // Common services: codecommit.amazonaws.com, cassandra.amazonaws.com
        if !service.ends_with(".amazonaws.com") {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid AWS service name: '{}'. Must end with .amazonaws.com",
                    service
                ),
            });
        }
        Ok(())
    }

    fn validate_path(&self, path: &str) -> Result<()> {
        // AWS paths must start and end with '/'
        if !path.starts_with('/') || !path.ends_with('/') {
            return Err(AmiError::InvalidParameter {
                message: format!(
                    "Invalid path: '{}'. AWS paths must start and end with '/'",
                    path
                ),
            });
        }
        Ok(())
    }

    fn generate_service_linked_role_name(
        &self,
        service_name: &str,
        custom_suffix: Option<&str>,
    ) -> String {
        // Extract service name from full principal (e.g., "elasticbeanstalk.amazonaws.com" -> "elasticbeanstalk")
        let service = Self::extract_service_name(service_name).unwrap_or(service_name);

        // Convert to PascalCase (e.g., "elastic-beanstalk" -> "ElasticBeanstalk")
        let pascal_name = Self::to_pascal_case(service);

        // AWS service-linked role naming: AWSServiceRoleFor<ServiceName>[_<Suffix>]
        if let Some(suffix) = custom_suffix {
            format!("AWSServiceRoleFor{}_{}", pascal_name, suffix)
        } else {
            format!("AWSServiceRoleFor{}", pascal_name)
        }
    }

    fn generate_service_linked_role_path(&self, service_name: &str) -> String {
        // AWS service-linked role path format: /aws-service-role/<service-name>/
        format!("/aws-service-role/{}/", service_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_provider_name() {
        let provider = AwsProvider::new();
        assert_eq!(provider.name(), "aws");
    }

    #[test]
    fn test_generate_user_arn() {
        let provider = AwsProvider::new();
        let arn =
            provider.generate_resource_identifier(ResourceType::User, "123456789012", "/", "alice");
        assert_eq!(arn, "arn:aws:iam::123456789012:user/alice");
    }

    #[test]
    fn test_generate_user_arn_with_path() {
        let provider = AwsProvider::new();
        let arn = provider.generate_resource_identifier(
            ResourceType::User,
            "123456789012",
            "/engineering/",
            "alice",
        );
        assert_eq!(arn, "arn:aws:iam::123456789012:user/engineering/alice");
    }

    #[test]
    fn test_generate_role_arn() {
        let provider = AwsProvider::new();
        let arn = provider.generate_resource_identifier(
            ResourceType::Role,
            "123456789012",
            "/service/",
            "EC2Role",
        );
        assert_eq!(arn, "arn:aws:iam::123456789012:role/service/EC2Role");
    }

    #[test]
    fn test_generate_user_id() {
        let provider = AwsProvider::new();
        let id = provider.generate_resource_id(ResourceType::User);
        assert!(id.starts_with("AIDA"));
        assert_eq!(id.len(), 21); // AIDA + 17 chars
    }

    #[test]
    fn test_generate_group_id() {
        let provider = AwsProvider::new();
        let id = provider.generate_resource_id(ResourceType::Group);
        assert!(id.starts_with("AGPA"));
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn test_generate_role_id() {
        let provider = AwsProvider::new();
        let id = provider.generate_resource_id(ResourceType::Role);
        assert!(id.starts_with("AROA"));
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn test_generate_access_key_id() {
        let provider = AwsProvider::new();
        let id = provider.generate_resource_id(ResourceType::AccessKey);
        assert!(id.starts_with("AKIA"));
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn test_resource_limits() {
        let provider = AwsProvider::new();
        let limits = provider.resource_limits();
        assert_eq!(limits.max_access_keys_per_user, 2);
        assert_eq!(limits.max_signing_certificates_per_user, 2);
        assert_eq!(limits.max_tags_per_resource, 50);
        assert_eq!(limits.session_duration_min, 3600);
        assert_eq!(limits.session_duration_max, 43200);
    }

    #[test]
    fn test_validate_aws_service_name() {
        let provider = AwsProvider::new();
        assert!(provider
            .validate_service_name("codecommit.amazonaws.com")
            .is_ok());
        assert!(provider
            .validate_service_name("cassandra.amazonaws.com")
            .is_ok());
        assert!(provider
            .validate_service_name("custom-service.amazonaws.com")
            .is_ok());
        assert!(provider.validate_service_name("invalid-service").is_err());
        assert!(provider
            .validate_service_name("service.google.com")
            .is_err());
    }

    #[test]
    fn test_validate_path() {
        let provider = AwsProvider::new();
        assert!(provider.validate_path("/").is_ok());
        assert!(provider.validate_path("/admin/").is_ok());
        assert!(provider.validate_path("/engineering/team/").is_ok());
        assert!(provider.validate_path("invalid").is_err());
        assert!(provider.validate_path("/invalid").is_err());
        assert!(provider.validate_path("invalid/").is_err());
    }

    #[test]
    fn test_validate_session_duration() {
        let provider = AwsProvider::new();
        assert!(provider.validate_session_duration(3600).is_ok()); // 1 hour (min)
        assert!(provider.validate_session_duration(7200).is_ok()); // 2 hours
        assert!(provider.validate_session_duration(43200).is_ok()); // 12 hours (max)
        assert!(provider.validate_session_duration(3599).is_err()); // Too short
        assert!(provider.validate_session_duration(43201).is_err()); // Too long
    }

    #[test]
    fn test_service_linked_role_name_simple() {
        let provider = AwsProvider::new();
        let name =
            provider.generate_service_linked_role_name("elasticbeanstalk.amazonaws.com", None);
        assert_eq!(name, "AWSServiceRoleForElasticbeanstalk");
    }

    #[test]
    fn test_service_linked_role_name_with_suffix() {
        let provider = AwsProvider::new();
        let name = provider.generate_service_linked_role_name("lex.amazonaws.com", Some("MyBot"));
        assert_eq!(name, "AWSServiceRoleForLex_MyBot");
    }

    #[test]
    fn test_service_linked_role_name_hyphenated() {
        let provider = AwsProvider::new();
        let name =
            provider.generate_service_linked_role_name("elastic-beanstalk.amazonaws.com", None);
        assert_eq!(name, "AWSServiceRoleForElasticBeanstalk");
    }

    #[test]
    fn test_service_linked_role_path() {
        let provider = AwsProvider::new();
        let path = provider.generate_service_linked_role_path("elasticbeanstalk.amazonaws.com");
        assert_eq!(path, "/aws-service-role/elasticbeanstalk.amazonaws.com/");
    }

    #[test]
    fn test_extract_service_name() {
        assert_eq!(
            AwsProvider::extract_service_name("elasticbeanstalk.amazonaws.com"),
            Some("elasticbeanstalk")
        );
        assert_eq!(
            AwsProvider::extract_service_name("lex.amazonaws.com"),
            Some("lex")
        );
        assert_eq!(
            AwsProvider::extract_service_name("simple-service"),
            Some("simple-service")
        );
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(
            AwsProvider::to_pascal_case("elasticbeanstalk"),
            "Elasticbeanstalk"
        );
        assert_eq!(
            AwsProvider::to_pascal_case("elastic-beanstalk"),
            "ElasticBeanstalk"
        );
        assert_eq!(
            AwsProvider::to_pascal_case("my-custom-service"),
            "MyCustomService"
        );
        assert_eq!(AwsProvider::to_pascal_case("lex"), "Lex");
    }

    #[test]
    fn test_custom_limits() {
        let limits = ResourceLimits {
            max_access_keys_per_user: 10,
            max_tags_per_resource: 100,
            ..Default::default()
        };
        let provider = AwsProvider::with_limits(limits);
        assert_eq!(provider.resource_limits().max_access_keys_per_user, 10);
        assert_eq!(provider.resource_limits().max_tags_per_resource, 100);
    }
}
