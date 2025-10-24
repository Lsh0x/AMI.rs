pub mod users;
pub mod access_keys;
pub mod passwords;
pub mod mfa_devices;
pub mod service_credentials;
pub mod groups;
pub mod roles;
pub mod policies;
pub mod permissions_boundaries;
pub mod policy_evaluation;
pub mod identity_providers;
pub mod server_certificates;
pub mod service_linked_roles;
pub mod tags;
pub mod reports;
pub mod signing_certificates;

use crate::error::Result;
use crate::types::{AmiResponse, PaginationParams};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// In-memory IAM client that simulates AWS IAM operations
#[derive(Debug, Clone)]
pub struct IamClient {
    // In-memory storage for all IAM resources
    users: HashMap<String, User>,
    groups: HashMap<String, Group>,
    roles: HashMap<String, Role>,
    policies: HashMap<String, Policy>,
    access_keys: HashMap<String, AccessKey>,
    mfa_devices: HashMap<String, MfaDevice>,
    // Add other resource storage as needed
}

impl IamClient {
    /// Create a new in-memory IAM client
    pub async fn new() -> Result<Self> {
        Ok(Self {
            users: HashMap::new(),
            groups: HashMap::new(),
            roles: HashMap::new(),
            policies: HashMap::new(),
            access_keys: HashMap::new(),
            mfa_devices: HashMap::new(),
        })
    }

    /// Create a new IAM client with custom configuration
    pub async fn with_config(_config: crate::types::AwsConfig) -> Result<Self> {
        // For in-memory implementation, config is not used
        Self::new().await
    }
}

// Common IAM resource types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_name: String,
    pub user_id: String,
    pub arn: String,
    pub path: String,
    pub create_date: chrono::DateTime<chrono::Utc>,
    pub password_last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub permissions_boundary: Option<String>,
    pub tags: Vec<crate::types::Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub group_name: String,
    pub group_id: String,
    pub arn: String,
    pub path: String,
    pub create_date: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<crate::types::Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub role_name: String,
    pub role_id: String,
    pub arn: String,
    pub path: String,
    pub create_date: chrono::DateTime<chrono::Utc>,
    pub assume_role_policy_document: String,
    pub description: Option<String>,
    pub max_session_duration: Option<i32>,
    pub permissions_boundary: Option<String>,
    pub tags: Vec<crate::types::Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub policy_name: String,
    pub policy_id: String,
    pub arn: String,
    pub path: String,
    pub default_version_id: String,
    pub attachment_count: i32,
    pub permissions_boundary_usage_count: i32,
    pub is_attachable: bool,
    pub description: Option<String>,
    pub create_date: chrono::DateTime<chrono::Utc>,
    pub update_date: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<crate::types::Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKey {
    pub user_name: String,
    pub access_key_id: String,
    pub status: String, // Active, Inactive
    pub create_date: chrono::DateTime<chrono::Utc>,
    pub secret_access_key: Option<String>, // Only returned on creation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaDevice {
    pub user_name: String,
    pub serial_number: String,
    pub enable_date: chrono::DateTime<chrono::Utc>,
}

// Re-export all sub-modules for easy access
pub use users::*;
pub use access_keys::*;
pub use passwords::*;
pub use mfa_devices::*;
pub use service_credentials::*;
pub use groups::*;
pub use roles::*;
pub use policies::*;
pub use permissions_boundaries::*;
pub use policy_evaluation::*;
pub use identity_providers::*;
pub use server_certificates::*;
pub use service_linked_roles::*;
pub use tags::*;
pub use reports::*;
pub use signing_certificates::*;
