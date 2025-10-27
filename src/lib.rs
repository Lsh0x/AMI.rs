//! WAMI - Who Am I: Multicloud Identity and Access Management library for Rust
//!
//! This library provides a multicloud implementation of Identity and Access Management (IAM),
//! Security Token Service (STS), and Single Sign-On Admin operations. It's designed to work
//! across multiple cloud providers (AWS, GCP, Azure, and custom platforms) and can be used
//! for testing, development, or as a unified identity layer for multicloud environments.
//!
//! ## Features
//!
//! - **🌐 Multicloud Support**: AWS, GCP, Azure, and custom identity providers
//! - **IAM Operations**: Complete user, group, role, and policy management
//! - **STS Operations**: Temporary credentials and identity inspection
//! - **SSO Admin Operations**: Permission sets, assignments, and instances
//! - **Pluggable Storage**: In-memory, database, or cloud-native backends
//! - **Async API**: All operations are asynchronous for better performance
//! - **Type Safety**: Strongly typed requests and responses
//!
//! ## Example
//!
//! ```rust
//! use wami::{MemoryIamClient, MemoryStsClient, MemorySsoAdminClient, InMemoryStore};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize logging to see account ID generation
//!     env_logger::init();
//!     
//!     // Initialize clients with in-memory storage
//!     let store = wami::create_memory_store();
//!     let account_id = wami::get_account_id_from_store(&store);
//!     println!("Using AWS account ID: {}", account_id);
//!     
//!     // Print AWS environment variables for export
//!     wami::print_aws_environment_variables(&store);
//!     
//!     let mut iam_client = MemoryIamClient::new(store.clone());
//!     let mut sts_client = MemoryStsClient::new(store.clone());
//!     let mut sso_client = MemorySsoAdminClient::new(store);
//!     
//!     // Get account ID from client
//!     let client_account_id = iam_client.account_id().await?;
//!     println!("Account ID from IAM client: {}", client_account_id);
//!     
//!     // Create a user
//!     let user_request = wami::CreateUserRequest {
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
pub mod iam;
pub mod provider;
pub mod sso_admin;
pub mod store;
pub mod sts;
pub mod tenant;
pub mod types;

// Re-export main types for convenience
pub use error::{AmiError, Result};
pub use types::{AmiResponse, AwsConfig, PaginationParams, PolicyDocument, PolicyStatement, Tag};

// Re-export store traits and implementations
pub use store::memory::InMemoryStore;
pub use store::{IamStore, SsoAdminStore, Store, StsStore};

// Re-export provider types
pub use provider::ProviderConfig;

// Re-export clients (now generic over stores)
pub use iam::IamClient;
pub use sso_admin::SsoAdminClient;
pub use sts::StsClient;
pub use tenant::TenantClient;

// Re-export IAM types
pub use iam::{
    AccessKey, Group, LoginProfile, MfaDevice, Policy, Role, ServerCertificate,
    ServerCertificateMetadata, ServiceSpecificCredential, ServiceSpecificCredentialMetadata, User,
};

// Re-export STS types
pub use sts::{CallerIdentity, Credentials, StsSession};

// Re-export SSO Admin types
pub use sso_admin::{
    AccountAssignment, Application, PermissionSet, SsoInstance, TrustedTokenIssuer,
};

// Re-export Tenant types
pub use tenant::{
    check_tenant_permission, BillingInfo, QuotaMode, Tenant, TenantAction, TenantId,
    TenantQuotas, TenantStatus, TenantType, TenantUsage,
};

// Re-export request/response types
pub use iam::access_key::{
    AccessKeyLastUsed, CreateAccessKeyRequest, ListAccessKeysRequest, ListAccessKeysResponse,
    UpdateAccessKeyRequest,
};
pub use iam::group::{
    CreateGroupRequest, ListGroupsRequest, ListGroupsResponse, UpdateGroupRequest,
};
pub use iam::login_profile::{
    CreateLoginProfileRequest, GetLoginProfileRequest, UpdateLoginProfileRequest,
};
pub use iam::mfa_device::{EnableMfaDeviceRequest, ListMfaDevicesRequest};
pub use iam::policy::{
    CreatePolicyRequest, ListPoliciesRequest, ListPoliciesResponse, UpdatePolicyRequest,
};
pub use iam::policy_evaluation::{
    ContextEntry, EvaluationResult, SimulateCustomPolicyRequest, SimulatePolicyResponse,
    SimulatePrincipalPolicyRequest, StatementMatch,
};
pub use iam::report::{
    AccountSummaryMap, CredentialReport, GenerateCredentialReportRequest,
    GenerateCredentialReportResponse, GetAccountSummaryRequest, GetAccountSummaryResponse,
    GetCredentialReportRequest, GetCredentialReportResponse, ReportState,
};
pub use iam::role::{CreateRoleRequest, ListRolesRequest, ListRolesResponse, UpdateRoleRequest};
pub use iam::server_certificate::{
    DeleteServerCertificateRequest, GetServerCertificateRequest, GetServerCertificateResponse,
    ListServerCertificatesRequest, ListServerCertificatesResponse, UpdateServerCertificateRequest,
    UploadServerCertificateRequest, UploadServerCertificateResponse,
};
pub use iam::service_credential::{
    CreateServiceSpecificCredentialRequest, CreateServiceSpecificCredentialResponse,
    DeleteServiceSpecificCredentialRequest, ListServiceSpecificCredentialsRequest,
    ListServiceSpecificCredentialsResponse, ResetServiceSpecificCredentialRequest,
    ResetServiceSpecificCredentialResponse, UpdateServiceSpecificCredentialRequest,
};
pub use iam::service_linked_role::{
    CreateServiceLinkedRoleRequest, CreateServiceLinkedRoleResponse,
    DeleteServiceLinkedRoleRequest, DeleteServiceLinkedRoleResponse, DeletionTaskFailureReason,
    DeletionTaskInfo, DeletionTaskStatus, GetServiceLinkedRoleDeletionStatusRequest,
    GetServiceLinkedRoleDeletionStatusResponse, RoleUsageType,
};
pub use iam::signing_certificate::{
    CertificateStatus, DeleteSigningCertificateRequest, ListSigningCertificatesRequest,
    ListSigningCertificatesResponse, SigningCertificate, UpdateSigningCertificateRequest,
    UploadSigningCertificateRequest, UploadSigningCertificateResponse,
};
pub use iam::tag::{ListResourceTagsRequest, TagResourceRequest, UntagResourceRequest};
pub use iam::user::{CreateUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest};
pub use sso_admin::{CreateAccountAssignmentRequest, CreatePermissionSetRequest};
pub use sts::{AssumeRoleRequest, GetFederationTokenRequest, GetSessionTokenRequest};

/// Initialize all clients with in-memory storage
pub fn initialize_clients_with_memory_store() -> (
    IamClient<InMemoryStore>,
    StsClient<InMemoryStore>,
    SsoAdminClient<InMemoryStore>,
    TenantClient<InMemoryStore>,
) {
    let store = InMemoryStore::new();
    let iam_client = IamClient::new(store.clone());
    let sts_client = StsClient::new(store.clone());
    let sso_client = SsoAdminClient::new(store.clone());
    let tenant_client = TenantClient::new(store, "admin@example.com".to_string());

    (iam_client, sts_client, sso_client, tenant_client)
}

/// Create a new in-memory store
pub fn create_memory_store() -> InMemoryStore {
    InMemoryStore::new()
}

// Note: Provider-specific functionality has been removed from the unified store.
// Resources now carry their own provider-specific information (ARNs, account IDs, etc.).
// If you need provider-specific functionality, use the client-level providers.

/// Type alias for convenience when using in-memory storage
pub type MemoryIamClient = IamClient<InMemoryStore>;
pub type MemoryStsClient = StsClient<InMemoryStore>;
pub type MemorySsoAdminClient = SsoAdminClient<InMemoryStore>;
pub type MemoryTenantClient = TenantClient<InMemoryStore>;
