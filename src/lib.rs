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
//! use ami::{MemoryIamClient, MemoryStsClient, MemorySsoAdminClient, InMemoryStore};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize logging to see account ID generation
//!     env_logger::init();
//!     
//!     // Initialize clients with in-memory storage
//!     let store = ami::create_memory_store();
//!     let account_id = ami::get_account_id_from_store(&store);
//!     println!("Using AWS account ID: {}", account_id);
//!     
//!     // Print AWS environment variables for export
//!     ami::print_aws_environment_variables(&store);
//!     
//!     let mut iam_client = MemoryIamClient::new(store);
//!     let mut sts_client = MemoryStsClient::new(store);
//!     let mut sso_client = MemorySsoAdminClient::new(store);
//!     
//!     // Get account ID from client
//!     let client_account_id = iam_client.account_id().await?;
//!     println!("Account ID from IAM client: {}", client_account_id);
//!     
//!     // Create a user
//!     let user_request = ami::CreateUserRequest {
//!         user_name: "test-user".to_string(),
//!         path: Some("/".to_string()),
//!         permissions_boundary: None,
//!         tags: None,
//!     };
//!     let user = iam_client.create_user(user_request).await?;
//!     println!("Created user: {:?}", user.data);
//!     println!("User ARN: {}", user.data.unwrap().arn);
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
pub mod store;
pub mod iam;
pub mod sts;
pub mod sso_admin;

// Re-export main types for convenience
pub use error::{AmiError, Result};
pub use types::{AmiResponse, AwsConfig, PaginationParams, Tag, PolicyDocument, PolicyStatement};

// Re-export store traits and implementations
pub use store::{IamStore, StsStore, SsoAdminStore, Store};
pub use store::in_memory::InMemoryStore;

// Re-export clients (now generic over stores)
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

/// Initialize all AWS clients with in-memory storage
pub fn initialize_clients_with_memory_store() -> (IamClient<InMemoryStore>, StsClient<InMemoryStore>, SsoAdminClient<InMemoryStore>) {
    let store = InMemoryStore::new();
    let iam_client = IamClient::new(store);
    let sts_client = StsClient::new(store);
    let sso_client = SsoAdminClient::new(store);
    
    (iam_client, sts_client, sso_client)
}

/// Create a new in-memory store
pub fn create_memory_store() -> InMemoryStore {
    InMemoryStore::new()
}

/// Create a new in-memory store with a specific account ID
pub fn create_memory_store_with_account_id(account_id: String) -> InMemoryStore {
    InMemoryStore::with_account_id(account_id)
}

/// Get the account ID from a store
pub fn get_account_id_from_store(store: &InMemoryStore) -> &str {
    store.account_id()
}

/// Get AWS environment variables from a store
pub fn get_aws_environment_variables(store: &InMemoryStore) -> std::collections::HashMap<String, String> {
    store.aws_environment_variables()
}

/// Print AWS environment variables from a store
pub fn print_aws_environment_variables(store: &InMemoryStore) {
    store.print_aws_environment_variables();
}

/// Type alias for convenience when using in-memory storage
pub type MemoryIamClient = IamClient<InMemoryStore>;
pub type MemoryStsClient = StsClient<InMemoryStore>;
pub type MemorySsoAdminClient = SsoAdminClient<InMemoryStore>;
