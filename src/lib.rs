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
//! use wami::store::memory::InMemoryWamiStore;
//! use wami::store::traits::UserStore;
//! use wami::provider::{AwsProvider, CloudProvider};
//! use wami::wami::identity::user::builder::build_user;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize logging
//!     env_logger::init();
//!     
//!     // Initialize store
//!     let mut store = InMemoryWamiStore::default();
//!     
//!     // Create provider
//!     let provider = AwsProvider::new();
//!     
//!     // Build a user using pure functions
//!     let user = build_user(
//!         "alice".to_string(),
//!         Some("/".to_string()),
//!         &provider,
//!         "123456789012",
//!     );
//!     
//!     // Store the user
//!     let created_user = store.create_user(user).await?;
//!     println!("Created user: {}", created_user.user_name);
//!     println!("User ARN: {}", created_user.arn);
//!     
//!     // Retrieve the user
//!     let retrieved = store.get_user("alice").await?;
//!     if let Some(user) = retrieved {
//!         println!("Retrieved user: {}", user.user_name);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod provider;
// pub mod service;  // Removed - will rebuild later with proper architecture
pub mod store;
pub mod types;
pub mod wami;

// Re-export main types for convenience
pub use error::{AmiError, Result};
pub use types::{AmiResponse, AwsConfig, PaginationParams, PolicyDocument, PolicyStatement, Tag};

// Re-export store traits and implementations
pub use store::memory::InMemoryStore;
pub use store::{SsoAdminStore, Store, StsStore, WamiStore};

// Re-export provider types
pub use provider::ProviderConfig;

// Re-export WAMI modules for convenience (Legacy compatibility)
pub use wami::{sso_admin, sts, tenant};

// Re-export identity types
pub use wami::identity::{Group, Role, User};

// Re-export credential types
pub use wami::credentials::{
    AccessKey, LoginProfile, MfaDevice, ServerCertificate, ServiceSpecificCredential,
};
// pub use wami::credentials::server_certificate::ServerCertificateMetadata; // TODO: fix path

// Re-export policy types
pub use wami::policies::Policy;

// Re-export report types
pub use wami::reports::CredentialReport;

// Re-export STS types
pub use wami::sts::{Credentials, StsSession};

// Re-export SSO Admin types (TODO: update when sso_admin is refactored)
// pub use wami::sso_admin::{AccountAssignment, Application, PermissionSet, SsoInstance, TrustedTokenIssuer};

// Re-export Tenant types
pub use wami::tenant::{
    check_tenant_permission, BillingInfo, QuotaMode, Tenant, TenantAction, TenantId, TenantQuotas,
    TenantStatus, TenantType, TenantUsage,
};

// Legacy IAM module alias
// #[deprecated(since = "0.2.0", note = "Use wami::identity, wami::credentials, etc. instead")]
// pub use wami::iam; // Temporarily removed to avoid warnings

// Re-export request/response types (updated paths)
pub use wami::credentials::access_key::{
    AccessKeyLastUsed, CreateAccessKeyRequest, ListAccessKeysRequest, ListAccessKeysResponse,
    UpdateAccessKeyRequest,
};
pub use wami::credentials::login_profile::{
    CreateLoginProfileRequest, GetLoginProfileRequest, UpdateLoginProfileRequest,
};
pub use wami::credentials::mfa_device::{EnableMfaDeviceRequest, ListMfaDevicesRequest};
pub use wami::credentials::server_certificate::{
    DeleteServerCertificateRequest, GetServerCertificateRequest, GetServerCertificateResponse,
    ListServerCertificatesRequest, ListServerCertificatesResponse, UpdateServerCertificateRequest,
    UploadServerCertificateRequest, UploadServerCertificateResponse,
};
pub use wami::credentials::service_credential::{
    CreateServiceSpecificCredentialRequest, CreateServiceSpecificCredentialResponse,
    DeleteServiceSpecificCredentialRequest, ListServiceSpecificCredentialsRequest,
    ListServiceSpecificCredentialsResponse, ResetServiceSpecificCredentialRequest,
    ResetServiceSpecificCredentialResponse, UpdateServiceSpecificCredentialRequest,
};
pub use wami::credentials::signing_certificate::{
    CertificateStatus, DeleteSigningCertificateRequest, ListSigningCertificatesRequest,
    ListSigningCertificatesResponse, SigningCertificate, UpdateSigningCertificateRequest,
    UploadSigningCertificateRequest, UploadSigningCertificateResponse,
};
pub use wami::identity::group::{
    CreateGroupRequest, ListGroupsRequest, ListGroupsResponse, UpdateGroupRequest,
};
pub use wami::identity::role::{
    CreateRoleRequest, ListRolesRequest, ListRolesResponse, UpdateRoleRequest,
};
pub use wami::identity::service_linked_role::{
    CreateServiceLinkedRoleRequest, CreateServiceLinkedRoleResponse,
    DeleteServiceLinkedRoleRequest, DeleteServiceLinkedRoleResponse, DeletionTaskFailureReason,
    DeletionTaskInfo, DeletionTaskStatus, GetServiceLinkedRoleDeletionStatusRequest,
    GetServiceLinkedRoleDeletionStatusResponse, RoleUsageType,
};
pub use wami::identity::user::{
    CreateUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest,
};
pub use wami::policies::evaluation::{
    ContextEntry, EvaluationResult, SimulateCustomPolicyRequest, SimulatePolicyResponse,
    SimulatePrincipalPolicyRequest, StatementMatch,
};
pub use wami::policies::policy::{
    CreatePolicyRequest, ListPoliciesRequest, ListPoliciesResponse, UpdatePolicyRequest,
};
pub use wami::reports::credential_report::{
    AccountSummaryMap, CredentialReport as CredentialReportType, GenerateCredentialReportRequest,
    GenerateCredentialReportResponse, GetAccountSummaryRequest, GetAccountSummaryResponse,
    GetCredentialReportRequest, GetCredentialReportResponse, ReportState,
};
pub use wami::tags::{ListResourceTagsRequest, TagResourceRequest, UntagResourceRequest};
// SSO Admin request types (to be re-exported when needed)
// pub use sso_admin::{CreateAccountAssignmentRequest, CreatePermissionSetRequest};
pub use sts::{AssumeRoleRequest, GetSessionTokenRequest};
// pub use sts::{GetFederationTokenRequest};  // Re-export when needed

// /// Initialize all clients with in-memory storage
// /// TEMPORARILY DISABLED during pure functions refactor
// pub fn initialize_clients_with_memory_store() -> (
//     IamClient<InMemoryStore>,
//     StsClient<InMemoryStore>,
//     SsoAdminClient<InMemoryStore>,
//     TenantClient<InMemoryStore>,
// ) {
//     let store = InMemoryStore::new();
//     let iam_client = IamClient::new(store.clone());
//     let sts_client = StsClient::new(store.clone());
//     let sso_client = SsoAdminClient::new(store.clone());
//     let tenant_client = TenantClient::new(store, "admin@example.com".to_string());
//
//     (iam_client, sts_client, sso_client, tenant_client)
// }

/// Create a new in-memory store
pub fn create_memory_store() -> InMemoryStore {
    InMemoryStore::new()
}

// Note: Provider-specific functionality has been removed from the unified store.
// Resources now carry their own provider-specific information (ARNs, account IDs, etc.).
// If you need provider-specific functionality, use the client-level providers.

// /// Type alias for convenience when using in-memory storage
// /// TODO: Rebuild these with service layer
// pub type MemoryIamClient = IamClient<InMemoryStore>;
// pub type MemoryStsClient = StsClient<InMemoryStore>;
// pub type MemorySsoAdminClient = SsoAdminClient<InMemoryStore>;
// pub type MemoryTenantClient = TenantClient<InMemoryStore>;
