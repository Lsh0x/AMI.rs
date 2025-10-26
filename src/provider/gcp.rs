//! Google Cloud Platform Provider Implementation
//!
//! This module contains the GCP-specific implementation of the CloudProvider trait.
//! Currently a stub implementation - full GCP support to be added in future versions.

use super::{CloudProvider, ResourceLimits, ResourceType};
use crate::error::Result;

/// Google Cloud Platform provider implementation
///
/// # Status
///
/// ⚠️ **Stub Implementation** - This is a placeholder for future GCP support.
///
/// # GCP Differences from AWS
///
/// - Uses numeric IDs instead of prefixed alphanumeric
/// - URN format: `projects/{project}/serviceAccounts/{email}`
/// - Different resource limits (e.g., 10 keys per service account)
/// - No concept of paths
/// - Service accounts instead of users
#[derive(Debug, Clone)]
pub struct GcpProvider {
    project_id: String,
    limits: ResourceLimits,
}

impl GcpProvider {
    /// Creates a new GCP provider for a specific project
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::provider::GcpProvider;
    ///
    /// let provider = GcpProvider::new("my-project-123");
    /// ```
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            limits: ResourceLimits {
                max_access_keys_per_user: 10, // GCP allows more keys
                max_service_credentials_per_user_per_service: 10,
                max_tags_per_resource: 64,  // GCP label limit
                session_duration_max: 3600, // GCP default: 1 hour
                ..Default::default()
            },
        }
    }
}

impl CloudProvider for GcpProvider {
    fn name(&self) -> &str {
        "gcp"
    }

    fn generate_resource_identifier(
        &self,
        resource_type: ResourceType,
        _account_id: &str,
        _path: &str,
        name: &str,
    ) -> String {
        // GCP uses different formats based on resource type
        match resource_type {
            ResourceType::User => {
                // Service accounts in GCP
                format!(
                    "projects/{}/serviceAccounts/{}@{}.iam.gserviceaccount.com",
                    self.project_id, name, self.project_id
                )
            }
            ResourceType::Role => {
                // Custom roles
                format!("projects/{}/roles/{}", self.project_id, name)
            }
            _ => {
                // Generic resource format
                format!("projects/{}/resources/{}", self.project_id, name)
            }
        }
    }

    fn generate_resource_id(&self, _resource_type: ResourceType) -> String {
        // GCP uses numeric IDs
        // In a real implementation, this would be assigned by GCP
        format!("{}", uuid::Uuid::new_v4().as_u128())
    }

    fn resource_limits(&self) -> &ResourceLimits {
        &self.limits
    }

    fn validate_service_name(&self, _service: &str) -> Result<()> {
        // GCP doesn't have the same service name concept as AWS
        // In a real implementation, we'd validate GCP API names
        Ok(())
    }

    fn validate_path(&self, _path: &str) -> Result<()> {
        // GCP doesn't use paths like AWS
        Ok(())
    }

    fn generate_service_linked_role_name(
        &self,
        service_name: &str,
        custom_suffix: Option<&str>,
    ) -> String {
        // GCP uses simpler naming
        if let Some(suffix) = custom_suffix {
            format!("{}-{}", service_name, suffix)
        } else {
            service_name.to_string()
        }
    }

    fn generate_service_linked_role_path(&self, _service_name: &str) -> String {
        // GCP doesn't use paths
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcp_provider_name() {
        let provider = GcpProvider::new("test-project");
        assert_eq!(provider.name(), "gcp");
    }

    #[test]
    fn test_generate_service_account_urn() {
        let provider = GcpProvider::new("my-project-123");
        let urn = provider.generate_resource_identifier(ResourceType::User, "", "", "alice");
        assert_eq!(
            urn,
            "projects/my-project-123/serviceAccounts/alice@my-project-123.iam.gserviceaccount.com"
        );
    }

    #[test]
    fn test_generate_role_urn() {
        let provider = GcpProvider::new("my-project-123");
        let urn = provider.generate_resource_identifier(ResourceType::Role, "", "", "CustomRole");
        assert_eq!(urn, "projects/my-project-123/roles/CustomRole");
    }

    #[test]
    fn test_generate_numeric_id() {
        let provider = GcpProvider::new("test-project");
        let id = provider.generate_resource_id(ResourceType::User);
        // Should be numeric (parseable as u128)
        assert!(id.parse::<u128>().is_ok());
    }

    #[test]
    fn test_gcp_resource_limits() {
        let provider = GcpProvider::new("test-project");
        let limits = provider.resource_limits();
        assert_eq!(limits.max_access_keys_per_user, 10); // GCP allows more
        assert_eq!(limits.max_tags_per_resource, 64); // GCP label limit
    }
}
