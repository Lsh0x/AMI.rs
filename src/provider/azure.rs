//! Microsoft Azure Provider Implementation
//!
//! This module contains the Azure-specific implementation of the CloudProvider trait.
//! Currently a stub implementation - full Azure support to be added in future versions.

use super::{CloudProvider, ResourceLimits, ResourceType};
use crate::error::Result;

/// Microsoft Azure provider implementation
///
/// # Status
///
/// ⚠️ **Stub Implementation** - This is a placeholder for future Azure support.
///
/// # Azure Differences from AWS
///
/// - Uses GUIDs for resource IDs
/// - Resource ID format: `/subscriptions/{sub}/resourceGroups/{rg}/providers/{ns}/{type}/{name}`
/// - Different resource limits
/// - Role-based access control (RBAC) model
#[derive(Debug, Clone)]
pub struct AzureProvider {
    subscription_id: String,
    resource_group: String,
    limits: ResourceLimits,
}

impl AzureProvider {
    /// Creates a new Azure provider
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::provider::AzureProvider;
    ///
    /// let provider = AzureProvider::new("sub-123", "my-rg");
    /// ```
    pub fn new(subscription_id: impl Into<String>, resource_group: impl Into<String>) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            resource_group: resource_group.into(),
            limits: ResourceLimits {
                max_tags_per_resource: 50, // Azure tag limit
                ..Default::default()
            },
        }
    }
}

impl CloudProvider for AzureProvider {
    fn name(&self) -> &str {
        "azure"
    }

    fn generate_resource_identifier(
        &self,
        resource_type: ResourceType,
        _account_id: &str,
        _path: &str,
        name: &str,
    ) -> String {
        let resource_type_name = match resource_type {
            ResourceType::User => "users",
            ResourceType::Group => "groups",
            ResourceType::Role => "roleAssignments",
            _ => "resources",
        };

        format!(
            "/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Authorization/{}/{}",
            self.subscription_id, self.resource_group, resource_type_name, name
        )
    }

    fn generate_resource_id(&self, _resource_type: ResourceType) -> String {
        // Azure uses GUIDs
        uuid::Uuid::new_v4().to_string()
    }

    fn resource_limits(&self) -> &ResourceLimits {
        &self.limits
    }

    fn validate_service_name(&self, _service: &str) -> Result<()> {
        Ok(())
    }

    fn validate_path(&self, _path: &str) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_provider_name() {
        let provider = AzureProvider::new("sub-123", "my-rg");
        assert_eq!(provider.name(), "azure");
    }

    #[test]
    fn test_generate_azure_resource_id_format() {
        let provider = AzureProvider::new("sub-123", "my-rg");
        let resource_id =
            provider.generate_resource_identifier(ResourceType::User, "", "", "alice");
        assert!(resource_id.contains("/subscriptions/sub-123/"));
        assert!(resource_id.contains("/resourceGroups/my-rg/"));
        assert!(resource_id.contains("Microsoft.Authorization"));
    }

    #[test]
    fn test_generate_guid() {
        let provider = AzureProvider::new("sub-123", "my-rg");
        let id = provider.generate_resource_id(ResourceType::User);
        // Should be a valid GUID
        assert!(uuid::Uuid::parse_str(&id).is_ok());
    }
}
