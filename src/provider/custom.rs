//! Custom Provider Implementation
//!
//! This module allows users to create their own custom provider implementations
//! with configurable ARN formats, ID prefixes, and resource limits.

use super::{CloudProvider, ResourceLimits, ResourceType};
use crate::error::Result;

/// Custom provider implementation for user-defined cloud platforms
///
/// # Example
///
/// ```rust
/// use wami::provider::{CustomProvider, ResourceLimits};
///
/// let provider = CustomProvider::builder()
///     .name("mycloud")
///     .arn_template("urn:{service}:{account}:{type}:{name}")
///     .id_prefix("MC")
///     .limits(ResourceLimits {
///         max_access_keys_per_user: 5,
///         max_tags_per_resource: 100,
///         ..Default::default()
///     })
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct CustomProvider {
    name: String,
    arn_template: String,
    id_prefix: String,
    limits: ResourceLimits,
}

impl CustomProvider {
    /// Creates a builder for custom provider configuration
    pub fn builder() -> CustomProviderBuilder {
        CustomProviderBuilder::default()
    }
}

impl CloudProvider for CustomProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_resource_identifier(
        &self,
        resource_type: ResourceType,
        account_id: &str,
        path: &str,
        name: &str,
    ) -> String {
        let resource_type_str = format!("{:?}", resource_type).to_lowercase();

        // Simple template replacement
        self.arn_template
            .replace("{account}", account_id)
            .replace("{type}", &resource_type_str)
            .replace("{path}", path)
            .replace("{name}", name)
            .replace("{service}", "identity")
    }

    fn generate_resource_id(&self, _resource_type: ResourceType) -> String {
        format!(
            "{}{}",
            self.id_prefix,
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        )
    }

    fn resource_limits(&self) -> &ResourceLimits {
        &self.limits
    }

    fn validate_service_name(&self, _service: &str) -> Result<()> {
        // Custom providers can define their own validation
        Ok(())
    }

    fn validate_path(&self, _path: &str) -> Result<()> {
        // Custom providers can define their own validation
        Ok(())
    }

    fn generate_service_linked_role_name(
        &self,
        service_name: &str,
        custom_suffix: Option<&str>,
    ) -> String {
        if let Some(suffix) = custom_suffix {
            format!("{}-{}", service_name, suffix)
        } else {
            service_name.to_string()
        }
    }

    fn generate_service_linked_role_path(&self, _service_name: &str) -> String {
        String::new()
    }
}

/// Builder for CustomProvider
#[derive(Debug, Clone, Default)]
pub struct CustomProviderBuilder {
    name: Option<String>,
    arn_template: Option<String>,
    id_prefix: Option<String>,
    limits: Option<ResourceLimits>,
}

impl CustomProviderBuilder {
    /// Sets the provider name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the ARN template
    ///
    /// Supported placeholders:
    /// - `{service}` - Service name (e.g., "identity")
    /// - `{account}` - Account ID
    /// - `{type}` - Resource type
    /// - `{path}` - Resource path
    /// - `{name}` - Resource name
    pub fn arn_template(mut self, template: impl Into<String>) -> Self {
        self.arn_template = Some(template.into());
        self
    }

    /// Sets the ID prefix
    pub fn id_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.id_prefix = Some(prefix.into());
        self
    }

    /// Sets custom resource limits
    pub fn limits(mut self, limits: ResourceLimits) -> Self {
        self.limits = Some(limits);
        self
    }

    /// Builds the CustomProvider
    pub fn build(self) -> CustomProvider {
        CustomProvider {
            name: self.name.unwrap_or_else(|| "custom".to_string()),
            arn_template: self
                .arn_template
                .unwrap_or_else(|| "urn:{service}:{account}:{type}/{path}{name}".to_string()),
            id_prefix: self.id_prefix.unwrap_or_else(|| "CUST".to_string()),
            limits: self.limits.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_provider_builder() {
        let provider = CustomProvider::builder()
            .name("mycloud")
            .arn_template("urn:mycloud:{account}:{type}:{name}")
            .id_prefix("MC")
            .build();

        assert_eq!(provider.name(), "mycloud");
    }

    #[test]
    fn test_custom_arn_generation() {
        let provider = CustomProvider::builder()
            .arn_template("resource:{account}/{type}/{name}")
            .build();

        let arn =
            provider.generate_resource_identifier(ResourceType::User, "tenant-123", "/", "alice");
        assert_eq!(arn, "resource:tenant-123/user/alice");
    }

    #[test]
    fn test_custom_id_generation() {
        let provider = CustomProvider::builder().id_prefix("TEST").build();

        let id = provider.generate_resource_id(ResourceType::User);
        assert!(id.starts_with("TEST"));
        assert_eq!(id.len(), 21); // TEST (4) + 17 random chars
    }

    #[test]
    fn test_custom_limits() {
        let limits = ResourceLimits {
            max_access_keys_per_user: 10,
            max_tags_per_resource: 200,
            ..Default::default()
        };

        let provider = CustomProvider::builder().limits(limits.clone()).build();

        assert_eq!(provider.resource_limits().max_access_keys_per_user, 10);
        assert_eq!(provider.resource_limits().max_tags_per_resource, 200);
    }

    #[test]
    fn test_default_values() {
        let provider = CustomProvider::builder().build();

        assert_eq!(provider.name(), "custom");
        assert!(provider
            .generate_resource_identifier(ResourceType::User, "123", "/", "alice")
            .contains("urn:"));
        assert!(provider
            .generate_resource_id(ResourceType::User)
            .starts_with("CUST"));
    }
}
