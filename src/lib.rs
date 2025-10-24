//! AMI.rs - AWS IAM, STS, and SSO Admin operations library for Rust
//!
//! This library provides an in-memory implementation of AWS Identity and Access Management (IAM),
//! Security Token Service (STS), and Single Sign-On Admin operations. It's designed to be
//! AWS IAM-compatible and can be used by other repositories for testing, development, or
//! as a lightweight alternative to AWS services.
//!
//! ## Features
//!
//! - **IAM Operations**: Complete user, group, role, and policy management
//! - **STS Operations**: Temporary credentials and identity inspection
//! - **SSO Admin Operations**: Permission sets, assignments, and instances
//! - **In-Memory Storage**: All operations are performed in-memory for fast access
//! - **Async API**: All operations are asynchronous for better performance
//! - **Type Safety**: Strongly typed requests and responses
//!
//! ## Example
//!
//! ```rust
//! use ami::{IamClient, StsClient, SsoAdminClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize clients
//!     let mut iam_client = IamClient::new().await?;
//!     let mut sts_client = StsClient::new().await?;
//!     let mut sso_client = SsoAdminClient::new().await?;
//!     
//!     // Create a user
//!     let user_request = ami::iam::users::CreateUserRequest {
//!         user_name: "test-user".to_string(),
//!         path: Some("/".to_string()),
//!         permissions_boundary: None,
//!         tags: None,
//!     };
//!     let user = iam_client.create_user(user_request).await?;
//!     println!("Created user: {:?}", user.data);
//!     
//!     // Get caller identity
//!     let identity = sts_client.get_caller_identity().await?;
//!     println!("Caller identity: {:?}", identity.data);
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;
pub mod iam;
pub mod sts;
pub mod sso_admin;

// Re-export main types for convenience
pub use error::{AmiError, Result};
pub use types::{AmiResponse, AwsConfig, PaginationParams, Tag, PolicyDocument, PolicyStatement};

// Re-export clients
pub use iam::IamClient;
pub use sts::StsClient;
pub use sso_admin::SsoAdminClient;

// Re-export IAM types
pub use iam::{User, Group, Role, Policy, AccessKey, MfaDevice};

// Re-export STS types
pub use sts::{StsSession, CallerIdentity, Credentials};

// Re-export SSO Admin types
pub use sso_admin::{PermissionSet, AccountAssignment, SsoInstance, Application, TrustedTokenIssuer};

// Re-export request/response types
pub use iam::users::{CreateUserRequest, UpdateUserRequest, ListUsersRequest, ListUsersResponse};
pub use iam::access_keys::{CreateAccessKeyRequest, UpdateAccessKeyRequest, ListAccessKeysRequest, ListAccessKeysResponse, AccessKeyLastUsed};
pub use iam::groups::{CreateGroupRequest, UpdateGroupRequest, ListGroupsRequest, ListGroupsResponse};
pub use sts::{AssumeRoleRequest, GetSessionTokenRequest, GetFederationTokenRequest};
pub use sso_admin::{CreatePermissionSetRequest, CreateAccountAssignmentRequest};

/// Initialize all AWS clients with default configuration
pub async fn initialize_clients() -> Result<(IamClient, StsClient, SsoAdminClient)> {
    let iam_client = IamClient::new().await?;
    let sts_client = StsClient::new().await?;
    let sso_client = SsoAdminClient::new().await?;
    
    Ok((iam_client, sts_client, sso_client))
}

/// Initialize all AWS clients with custom configuration
pub async fn initialize_clients_with_config(config: AwsConfig) -> Result<(IamClient, StsClient, SsoAdminClient)> {
    let iam_client = IamClient::with_config(config.clone()).await?;
    let sts_client = StsClient::with_config(config.clone()).await?;
    let sso_client = SsoAdminClient::with_config(config).await?;
    
    Ok((iam_client, sts_client, sso_client))
}
