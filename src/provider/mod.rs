//! Cloud Provider Abstraction
//!
//! This module provides abstractions for different cloud providers (AWS, GCP, Azure, custom).
//! It allows the IAM system to work across multiple cloud platforms by abstracting
//! provider-specific details like ARN formats, ID generation, and resource limits.
//!
//! # Example
//!
//! ```rust
//! use wami::provider::{AwsProvider, GcpProvider, CloudProvider};
//!
//! // Use AWS provider (default)
//! let aws = AwsProvider::default();
//! let user_arn = aws.generate_resource_identifier(
//!     wami::provider::ResourceType::User,
//!     "123456789012",
//!     "/",
//!     "alice"
//! );
//! // → "arn:aws:iam::123456789012:user/alice"
//!
//! // Use GCP provider
//! let gcp = GcpProvider::new("my-project-123");
//! let user_arn = gcp.generate_resource_identifier(
//!     wami::provider::ResourceType::User,
//!     "",
//!     "",
//!     "alice"
//! );
//! // → "projects/my-project-123/serviceAccounts/alice@my-project-123.iam.gserviceaccount.com"
//! ```

pub mod aws;
pub mod azure;
pub mod custom;
pub mod gcp;

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Resource type enumeration for cloud resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// IAM User
    User,
    /// IAM Group
    Group,
    /// IAM Role
    Role,
    /// IAM Policy
    Policy,
    /// Access Key for programmatic access
    AccessKey,
    /// Server Certificate (SSL/TLS)
    ServerCertificate,
    /// Service-specific Credential
    ServiceCredential,
    /// MFA Device
    MfaDevice,
    /// Signing Certificate
    SigningCertificate,
}

/// Resource limits configuration per cloud provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum number of access keys per user
    pub max_access_keys_per_user: usize,
    /// Maximum number of signing certificates per user
    pub max_signing_certificates_per_user: usize,
    /// Maximum number of service credentials per user per service
    pub max_service_credentials_per_user_per_service: usize,
    /// Maximum number of tags per resource
    pub max_tags_per_resource: usize,
    /// Maximum number of MFA devices per user
    pub max_mfa_devices_per_user: usize,
    /// Minimum session duration in seconds
    pub session_duration_min: i32,
    /// Maximum session duration in seconds
    pub session_duration_max: i32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        // AWS defaults
        Self {
            max_access_keys_per_user: 2,
            max_signing_certificates_per_user: 2,
            max_service_credentials_per_user_per_service: 2,
            max_tags_per_resource: 50,
            max_mfa_devices_per_user: 8,
            session_duration_min: 3600,  // 1 hour
            session_duration_max: 43200, // 12 hours
        }
    }
}

/// Cloud provider trait for abstracting provider-specific logic
///
/// This trait allows the library to work with different cloud providers
/// by abstracting provider-specific details like ARN formats, ID generation,
/// resource limits, and validation rules.
pub trait CloudProvider: Send + Sync + std::fmt::Debug {
    /// Returns the provider name (e.g., "aws", "gcp", "azure", "custom")
    fn name(&self) -> &str;

    /// Generates a resource identifier (ARN, URN, Resource ID, etc.)
    ///
    /// # Arguments
    ///
    /// * `resource_type` - The type of resource
    /// * `account_id` - The account/project/subscription identifier
    /// * `path` - The resource path (may be empty for providers that don't use paths)
    /// * `name` - The resource name
    ///
    /// # Returns
    ///
    /// A fully qualified resource identifier in the provider's format
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::provider::{AwsProvider, CloudProvider, ResourceType};
    ///
    /// let provider = AwsProvider::default();
    /// let arn = provider.generate_resource_identifier(
    ///     ResourceType::User,
    ///     "123456789012",
    ///     "/engineering/",
    ///     "alice"
    /// );
    /// assert_eq!(arn, "arn:aws:iam::123456789012:user/engineering/alice");
    /// ```
    fn generate_resource_identifier(
        &self,
        resource_type: ResourceType,
        account_id: &str,
        path: &str,
        name: &str,
    ) -> String;

    /// Generates a unique resource ID
    ///
    /// # Arguments
    ///
    /// * `resource_type` - The type of resource to generate an ID for
    ///
    /// # Returns
    ///
    /// A unique identifier in the provider's format
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::provider::{AwsProvider, CloudProvider, ResourceType};
    ///
    /// let provider = AwsProvider::default();
    /// let id = provider.generate_resource_id(ResourceType::User);
    /// assert!(id.starts_with("AIDA")); // AWS format
    /// assert_eq!(id.len(), 21); // AIDA + 17 chars
    /// ```
    fn generate_resource_id(&self, resource_type: ResourceType) -> String;

    /// Returns the resource limits for this provider
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::provider::{AwsProvider, CloudProvider};
    ///
    /// let provider = AwsProvider::default();
    /// let limits = provider.resource_limits();
    /// assert_eq!(limits.max_access_keys_per_user, 2); // AWS limit
    /// ```
    fn resource_limits(&self) -> &ResourceLimits;

    /// Validates a service name for service-specific credentials
    ///
    /// # Arguments
    ///
    /// * `service` - The service name to validate
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, Err otherwise
    #[allow(clippy::result_large_err)]
    fn validate_service_name(&self, service: &str) -> Result<()>;

    /// Validates a path format
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, Err otherwise
    #[allow(clippy::result_large_err)]
    fn validate_path(&self, path: &str) -> Result<()>;

    /// Validates a session duration
    ///
    /// # Arguments
    ///
    /// * `duration` - The session duration in seconds
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, Err otherwise
    ///
    /// # Default Implementation
    ///
    /// Checks against the provider's resource limits
    #[allow(clippy::result_large_err)]
    fn validate_session_duration(&self, duration: i32) -> Result<()> {
        let limits = self.resource_limits();
        if duration < limits.session_duration_min || duration > limits.session_duration_max {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "Session duration must be between {} and {} seconds",
                    limits.session_duration_min, limits.session_duration_max
                ),
            });
        }
        Ok(())
    }

    /// Generates a service-linked role name
    ///
    /// # Arguments
    ///
    /// * `service_name` - The service name (e.g., "elasticbeanstalk.amazonaws.com")
    /// * `custom_suffix` - Optional custom suffix for the role name
    ///
    /// # Returns
    ///
    /// A service-linked role name in the provider's format
    fn generate_service_linked_role_name(
        &self,
        service_name: &str,
        custom_suffix: Option<&str>,
    ) -> String;

    /// Generates a service-linked role path
    ///
    /// # Arguments
    ///
    /// * `service_name` - The service name
    ///
    /// # Returns
    ///
    /// A service-linked role path in the provider's format
    fn generate_service_linked_role_path(&self, service_name: &str) -> String;
}

// Re-export provider implementations
pub use aws::AwsProvider;
pub use azure::AzureProvider;
pub use custom::CustomProvider;
pub use gcp::GcpProvider;
