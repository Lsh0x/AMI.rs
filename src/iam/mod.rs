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

pub mod access_key;
pub mod group;
pub mod identity_providers;
pub mod mfa_devices;
pub mod passwords;
pub mod permissions_boundaries;
pub mod policy;
pub mod policy_evaluation;
pub mod reports;
pub mod role;
pub mod server_certificates;
pub mod service_credentials;
pub mod service_linked_roles;
pub mod signing_certificates;
pub mod tags;
pub mod user;

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
    pub async fn iam_store(&mut self) -> Result<&mut S::IamStore> {
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

// User is now defined in iam::user::model
pub use user::User;
// Group is now defined in iam::group::model
pub use group::Group;
// Role is now defined in iam::role::model
pub use role::Role;
// Policy is now defined in iam::policy::model
pub use policy::Policy;
// AccessKey is now defined in iam::access_key::model
pub use access_key::AccessKey;

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
pub use access_key::{
    AccessKeyLastUsed, CreateAccessKeyRequest, ListAccessKeysRequest, ListAccessKeysResponse,
    UpdateAccessKeyRequest,
};
pub use server_certificates::{ServerCertificate, ServerCertificateMetadata};
pub use service_credentials::{ServiceSpecificCredential, ServiceSpecificCredentialMetadata};
pub use signing_certificates::{CertificateStatus, SigningCertificate};
// User operations are in iam::user::operations
// Group operations are in iam::group::operations
// Role operations are in iam::role::operations
// Policy operations are in iam::policy::operations
// Re-export request types for convenience
pub use group::{CreateGroupRequest, ListGroupsRequest, ListGroupsResponse, UpdateGroupRequest};
pub use policy::{
    CreatePolicyRequest, ListPoliciesRequest, ListPoliciesResponse, UpdatePolicyRequest,
};
pub use role::{CreateRoleRequest, ListRolesRequest, ListRolesResponse, UpdateRoleRequest};
pub use user::{CreateUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest};
