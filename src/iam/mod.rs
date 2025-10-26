//! AWS Identity and Access Management (IAM) Operations
//!
//! This module provides comprehensive IAM functionality for managing AWS users, groups, roles,
//! policies, and access credentials. It offers a complete, type-safe API for IAM operations.
//!
//! # Overview
//!
//! The IAM module is organized into several sub-modules:
//!
//! - [`users`] - User management operations
//! - [`access_keys`] - Access key management for programmatic access
//! - [`passwords`] - Password and login profile management
//! - [`mfa_devices`] - Multi-factor authentication device management
//! - [`groups`] - User group management
//! - [`roles`] - IAM role management for AWS services and federated users
//! - [`policies`] - Managed and inline policy management
//! - [`permissions_boundaries`] - Permissions boundary management
//! - [`policy_evaluation`] - Policy simulation and evaluation
//! - [`identity_providers`] - SAML and OIDC identity provider management
//! - [`server_certificates`] - SSL/TLS certificate management
//! - [`service_linked_roles`] - Service-linked role management
//! - [`service_credentials`] - Service-specific credential management
//! - [`signing_certificates`] - X.509 signing certificate management
//! - [`tags`] - Resource tagging operations
//! - [`reports`] - Credential and access reports
//!
//! # Example
//!
//! ```rust,ignore
//! use wami::{MemoryIamClient, CreateUserRequest, CreateAccessKeyRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize the IAM client with in-memory storage
//! let store = wami::create_memory_store();
//! let mut iam_client = MemoryIamClient::new(store);
//!
//! // Create a new IAM user
//! let user_request = CreateUserRequest {
//!     user_name: "engineering-user".to_string(),
//!     path: Some("/engineering/".to_string()),
//!     permissions_boundary: None,
//!     tags: None,
//! };
//! let user_response = iam_client.create_user(user_request).await?;
//! let user = user_response.data.unwrap();
//! println!("Created user: {}", user.arn);
//!
//! // Create access keys for the user
//! let key_request = CreateAccessKeyRequest {
//!     user_name: "engineering-user".to_string(),
//! };
//! let key_response = iam_client.create_access_key(key_request).await?;
//! let access_key = key_response.data.unwrap();
//! println!("Access Key ID: {}", access_key.access_key_id);
//! # Ok(())
//! # }
//! ```

pub mod access_keys;
pub mod groups;
pub mod identity_providers;
pub mod mfa_devices;
pub mod passwords;
pub mod permissions_boundaries;
pub mod policies;
pub mod policy_evaluation;
pub mod reports;
pub mod roles;
pub mod server_certificates;
pub mod service_credentials;
pub mod service_linked_roles;
pub mod signing_certificates;
pub mod tags;
pub mod users;

use crate::error::Result;
use crate::store::{IamStore, Store};
use serde::{Deserialize, Serialize};

/// IAM client for managing AWS Identity and Access Management resources
///
/// The IAM client provides methods for managing users, groups, roles, policies,
/// and other IAM resources. It works with any store implementation that implements
/// the [`Store`] trait.
///
/// # Type Parameters
///
/// * `S` - The store implementation (e.g., [`InMemoryStore`](crate::store::in_memory::InMemoryStore))
///
/// # Example
///
/// ```rust
/// use wami::{MemoryIamClient, CreateUserRequest};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let store = wami::create_memory_store();
/// let mut iam_client = MemoryIamClient::new(store);
///
/// let request = CreateUserRequest {
///     user_name: "alice".to_string(),
///     path: Some("/".to_string()),
///     permissions_boundary: None,
///     tags: None,
/// };
///
/// let response = iam_client.create_user(request).await?;
/// println!("Created user: {:?}", response.data);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct IamClient<S: Store> {
    store: S,
}

impl<S: Store> IamClient<S> {
    /// Creates a new IAM client with the specified store
    ///
    /// # Arguments
    ///
    /// * `store` - The storage backend for IAM resources
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{IamClient, InMemoryStore};
    ///
    /// let store = InMemoryStore::new();
    /// let iam_client = IamClient::new(store);
    /// ```
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Get mutable reference to the IAM store
    async fn iam_store(&mut self) -> Result<&mut S::IamStore> {
        self.store.iam_store().await
    }

    /// Returns the AWS account ID associated with this client
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::MemoryIamClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// let account_id = iam_client.account_id().await?;
    /// println!("Account ID: {}", account_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn account_id(&mut self) -> Result<String> {
        let store = self.iam_store().await?;
        Ok(store.account_id().to_string())
    }
}

// Common IAM resource types

/// Represents an IAM user
///
/// An IAM user is an entity that represents a person or service that interacts with AWS.
///
/// # Example
///
/// ```rust
/// use wami::User;
/// use chrono::Utc;
///
/// let user = User {
///     user_name: "alice".to_string(),
///     user_id: "AIDACKCEVSQ6C2EXAMPLE".to_string(),
///     arn: "arn:aws:iam::123456789012:user/alice".to_string(),
///     path: "/".to_string(),
///     create_date: Utc::now(),
///     password_last_used: None,
///     permissions_boundary: None,
///     tags: vec![],
///     wami_arn: "arn:wami:iam::123456789012:user/alice".to_string(),
///     providers: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The friendly name identifying the user
    pub user_name: String,
    /// The stable and unique identifier for the user
    pub user_id: String,
    /// The Amazon Resource Name (ARN) that identifies the user
    pub arn: String,
    /// The path to the user
    pub path: String,
    /// The date and time when the user was created
    pub create_date: chrono::DateTime<chrono::Utc>,
    /// The date and time when the user's password was last used
    pub password_last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// The ARN of the policy used to set the permissions boundary
    pub permissions_boundary: Option<String>,
    /// A list of tags associated with the user
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Represents an IAM group
///
/// A group is a collection of IAM users. Groups let you specify permissions for multiple users.
///
/// # Example
///
/// ```rust
/// use wami::Group;
/// use chrono::Utc;
///
/// let group = Group {
///     group_name: "Developers".to_string(),
///     group_id: "AGPACKCEVSQ6C2EXAMPLE".to_string(),
///     arn: "arn:aws:iam::123456789012:group/Developers".to_string(),
///     path: "/engineering/".to_string(),
///     create_date: Utc::now(),
///     tags: vec![],
///     wami_arn: "arn:wami:iam::123456789012:group/Developers".to_string(),
///     providers: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// The friendly name identifying the group
    pub group_name: String,
    /// The stable and unique identifier for the group
    pub group_id: String,
    /// The Amazon Resource Name (ARN) that identifies the group
    pub arn: String,
    /// The path to the group
    pub path: String,
    /// The date and time when the group was created
    pub create_date: chrono::DateTime<chrono::Utc>,
    /// A list of tags associated with the group
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Represents an IAM role
///
/// An IAM role is similar to a user but is intended to be assumable by anyone who needs it.
///
/// # Example
///
/// ```rust
/// use wami::Role;
/// use chrono::Utc;
///
/// let role = Role {
///     role_name: "EC2-S3-Access".to_string(),
///     role_id: "AIDACKCEVSQ6C2EXAMPLE".to_string(),
///     arn: "arn:aws:iam::123456789012:role/EC2-S3-Access".to_string(),
///     path: "/".to_string(),
///     create_date: Utc::now(),
///     assume_role_policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
///     description: Some("Allows EC2 instances to access S3".to_string()),
///     max_session_duration: Some(3600),
///     permissions_boundary: None,
///     tags: vec![],
///     wami_arn: "arn:wami:iam::123456789012:role/EC2-S3-Access".to_string(),
///     providers: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// The friendly name identifying the role
    pub role_name: String,
    /// The stable and unique identifier for the role
    pub role_id: String,
    /// The Amazon Resource Name (ARN) that identifies the role
    pub arn: String,
    /// The path to the role
    pub path: String,
    /// The date and time when the role was created
    pub create_date: chrono::DateTime<chrono::Utc>,
    /// The trust policy that grants permission to assume the role
    pub assume_role_policy_document: String,
    /// A description of the role
    pub description: Option<String>,
    /// The maximum session duration in seconds
    pub max_session_duration: Option<i32>,
    /// The ARN of the policy used to set the permissions boundary
    pub permissions_boundary: Option<String>,
    /// A list of tags associated with the role
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Represents an IAM managed policy
///
/// A managed policy is a standalone policy that can be attached to multiple users, groups, and roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// The friendly name identifying the policy
    pub policy_name: String,
    /// The stable and unique identifier for the policy
    pub policy_id: String,
    /// The Amazon Resource Name (ARN) that identifies the policy
    pub arn: String,
    /// The path to the policy
    pub path: String,
    /// The identifier for the default version of the policy
    pub default_version_id: String,
    /// The policy document in JSON format
    pub policy_document: String,
    /// The number of entities (users, groups, and roles) that the policy is attached to
    pub attachment_count: i32,
    /// The number of entities that have the policy set as a permissions boundary
    pub permissions_boundary_usage_count: i32,
    /// Whether the policy can be attached to users, groups, or roles
    pub is_attachable: bool,
    /// A friendly description of the policy
    pub description: Option<String>,
    /// The date and time when the policy was created
    pub create_date: chrono::DateTime<chrono::Utc>,
    /// The date and time when the policy was last updated
    pub update_date: chrono::DateTime<chrono::Utc>,
    /// A list of tags associated with the policy
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Represents an IAM access key
///
/// Access keys are long-term credentials used to sign programmatic requests to AWS.
///
/// # Example
///
/// ```rust
/// use wami::AccessKey;
/// use chrono::Utc;
///
/// let access_key = AccessKey {
///     user_name: "alice".to_string(),
///     access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
///     status: "Active".to_string(),
///     create_date: Utc::now(),
///     secret_access_key: Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string()),
///     wami_arn: "arn:wami:iam::123456789012:access-key/AKIAIOSFODNN7EXAMPLE".to_string(),
///     providers: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKey {
    /// The name of the IAM user that the key is associated with
    pub user_name: String,
    /// The ID for this access key
    pub access_key_id: String,
    /// The status of the access key: Active or Inactive
    pub status: String,
    /// The date when the access key was created
    pub create_date: chrono::DateTime<chrono::Utc>,
    /// The secret key used to sign requests (only provided when creating the key)
    pub secret_access_key: Option<String>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Represents an MFA (Multi-Factor Authentication) device
///
/// # Example
///
/// ```rust
/// use wami::MfaDevice;
/// use chrono::Utc;
///
/// let mfa_device = MfaDevice {
///     user_name: "alice".to_string(),
///     serial_number: "arn:aws:iam::123456789012:mfa/alice".to_string(),
///     enable_date: Utc::now(),
///     wami_arn: "arn:wami:iam::123456789012:mfa/alice".to_string(),
///     providers: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaDevice {
    /// The user with whom the MFA device is associated
    pub user_name: String,
    /// The serial number that uniquely identifies the MFA device
    pub serial_number: String,
    /// The date when the MFA device was enabled
    pub enable_date: chrono::DateTime<chrono::Utc>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Represents a login profile (console password) for an IAM user
///
/// # Example
///
/// ```rust
/// use wami::LoginProfile;
/// use chrono::Utc;
///
/// let profile = LoginProfile {
///     user_name: "alice".to_string(),
///     create_date: Utc::now(),
///     password_reset_required: false,
///     wami_arn: "arn:wami:iam::123456789012:login-profile/alice".to_string(),
///     providers: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginProfile {
    /// The user with whom the login profile is associated
    pub user_name: String,
    /// The date when the login profile was created
    pub create_date: chrono::DateTime<chrono::Utc>,
    /// Whether the user must reset their password on next sign-in
    pub password_reset_required: bool,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

// Re-export all sub-modules for easy access
pub use access_keys::*;
pub use groups::*;
pub use server_certificates::{ServerCertificate, ServerCertificateMetadata};
pub use service_credentials::{ServiceSpecificCredential, ServiceSpecificCredentialMetadata};
pub use signing_certificates::{CertificateStatus, SigningCertificate};
pub use users::*;
