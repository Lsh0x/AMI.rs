pub mod memory;
pub mod memory_sts_sso;
pub mod in_memory;

use crate::error::Result;
use crate::types::{PaginationParams, Tag};
use crate::iam::{User, Group, Role, Policy, AccessKey, MfaDevice};
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for IAM data storage operations
/// This allows easy swapping between in-memory, database, or AWS backends
#[async_trait]
pub trait IamStore: Send + Sync {
    /// Get the account ID for this store
    fn account_id(&self) -> &str;
    // User operations
    async fn create_user(&mut self, user: User) -> Result<User>;
    async fn get_user(&self, user_name: &str) -> Result<Option<User>>;
    async fn update_user(&mut self, user: User) -> Result<User>;
    async fn delete_user(&mut self, user_name: &str) -> Result<()>;
    async fn list_users(&self, path_prefix: Option<&str>, pagination: Option<&PaginationParams>) -> Result<(Vec<User>, bool, Option<String>)>;
    async fn tag_user(&mut self, user_name: &str, tags: Vec<Tag>) -> Result<()>;
    async fn untag_user(&mut self, user_name: &str, tag_keys: Vec<String>) -> Result<()>;
    async fn list_user_tags(&self, user_name: &str) -> Result<Vec<Tag>>;

    // Access key operations
    async fn create_access_key(&mut self, access_key: AccessKey) -> Result<AccessKey>;
    async fn get_access_key(&self, access_key_id: &str) -> Result<Option<AccessKey>>;
    async fn update_access_key(&mut self, access_key: AccessKey) -> Result<AccessKey>;
    async fn delete_access_key(&mut self, access_key_id: &str) -> Result<()>;
    async fn list_access_keys(&self, user_name: &str, pagination: Option<&PaginationParams>) -> Result<(Vec<AccessKey>, bool, Option<String>)>;

    // Group operations
    async fn create_group(&mut self, group: Group) -> Result<Group>;
    async fn get_group(&self, group_name: &str) -> Result<Option<Group>>;
    async fn update_group(&mut self, group: Group) -> Result<Group>;
    async fn delete_group(&mut self, group_name: &str) -> Result<()>;
    async fn list_groups(&self, path_prefix: Option<&str>, pagination: Option<&PaginationParams>) -> Result<(Vec<Group>, bool, Option<String>)>;
    async fn list_groups_for_user(&self, user_name: &str) -> Result<Vec<Group>>;
    async fn add_user_to_group(&mut self, group_name: &str, user_name: &str) -> Result<()>;
    async fn remove_user_from_group(&mut self, group_name: &str, user_name: &str) -> Result<()>;

    // Role operations
    async fn create_role(&mut self, role: Role) -> Result<Role>;
    async fn get_role(&self, role_name: &str) -> Result<Option<Role>>;
    async fn update_role(&mut self, role: Role) -> Result<Role>;
    async fn delete_role(&mut self, role_name: &str) -> Result<()>;
    async fn list_roles(&self, path_prefix: Option<&str>, pagination: Option<&PaginationParams>) -> Result<(Vec<Role>, bool, Option<String>)>;

    // Policy operations
    async fn create_policy(&mut self, policy: Policy) -> Result<Policy>;
    async fn get_policy(&self, policy_arn: &str) -> Result<Option<Policy>>;
    async fn update_policy(&mut self, policy: Policy) -> Result<Policy>;
    async fn delete_policy(&mut self, policy_arn: &str) -> Result<()>;
    async fn list_policies(&self, scope: Option<&str>, pagination: Option<&PaginationParams>) -> Result<(Vec<Policy>, bool, Option<String>)>;

    // MFA device operations
    async fn create_mfa_device(&mut self, mfa_device: MfaDevice) -> Result<MfaDevice>;
    async fn get_mfa_device(&self, serial_number: &str) -> Result<Option<MfaDevice>>;
    async fn delete_mfa_device(&mut self, serial_number: &str) -> Result<()>;
    async fn list_mfa_devices(&self, user_name: &str) -> Result<Vec<MfaDevice>>;
}

/// Trait for STS data storage operations
#[async_trait]
pub trait StsStore: Send + Sync {
    /// Get the account ID for this store
    fn account_id(&self) -> &str;
    async fn create_session(&mut self, session: crate::sts::StsSession) -> Result<crate::sts::StsSession>;
    async fn get_session(&self, session_token: &str) -> Result<Option<crate::sts::StsSession>>;
    async fn delete_session(&mut self, session_token: &str) -> Result<()>;
    async fn list_sessions(&self, user_id: Option<&str>) -> Result<Vec<crate::sts::StsSession>>;
    
    async fn create_identity(&mut self, identity: crate::sts::CallerIdentity) -> Result<crate::sts::CallerIdentity>;
    async fn get_identity(&self, arn: &str) -> Result<Option<crate::sts::CallerIdentity>>;
    async fn list_identities(&self) -> Result<Vec<crate::sts::CallerIdentity>>;
}

/// Trait for SSO Admin data storage operations
#[async_trait]
pub trait SsoAdminStore: Send + Sync {
    async fn create_permission_set(&mut self, permission_set: crate::sso_admin::PermissionSet) -> Result<crate::sso_admin::PermissionSet>;
    async fn get_permission_set(&self, permission_set_arn: &str) -> Result<Option<crate::sso_admin::PermissionSet>>;
    async fn update_permission_set(&mut self, permission_set: crate::sso_admin::PermissionSet) -> Result<crate::sso_admin::PermissionSet>;
    async fn delete_permission_set(&mut self, permission_set_arn: &str) -> Result<()>;
    async fn list_permission_sets(&self, instance_arn: &str) -> Result<Vec<crate::sso_admin::PermissionSet>>;

    async fn create_account_assignment(&mut self, assignment: crate::sso_admin::AccountAssignment) -> Result<crate::sso_admin::AccountAssignment>;
    async fn get_account_assignment(&self, assignment_id: &str) -> Result<Option<crate::sso_admin::AccountAssignment>>;
    async fn delete_account_assignment(&mut self, assignment_id: &str) -> Result<()>;
    async fn list_account_assignments(&self, account_id: &str, permission_set_arn: &str) -> Result<Vec<crate::sso_admin::AccountAssignment>>;

    async fn create_instance(&mut self, instance: crate::sso_admin::SsoInstance) -> Result<crate::sso_admin::SsoInstance>;
    async fn get_instance(&self, instance_arn: &str) -> Result<Option<crate::sso_admin::SsoInstance>>;
    async fn list_instances(&self) -> Result<Vec<crate::sso_admin::SsoInstance>>;

    async fn create_application(&mut self, application: crate::sso_admin::Application) -> Result<crate::sso_admin::Application>;
    async fn get_application(&self, application_arn: &str) -> Result<Option<crate::sso_admin::Application>>;
    async fn list_applications(&self, instance_arn: &str) -> Result<Vec<crate::sso_admin::Application>>;

    async fn create_trusted_token_issuer(&mut self, issuer: crate::sso_admin::TrustedTokenIssuer) -> Result<crate::sso_admin::TrustedTokenIssuer>;
    async fn get_trusted_token_issuer(&self, issuer_arn: &str) -> Result<Option<crate::sso_admin::TrustedTokenIssuer>>;
    async fn delete_trusted_token_issuer(&mut self, issuer_arn: &str) -> Result<()>;
    async fn list_trusted_token_issuers(&self, instance_arn: &str) -> Result<Vec<crate::sso_admin::TrustedTokenIssuer>>;
}

/// Generic store trait that can be implemented by any backend
#[async_trait]
pub trait Store: Send + Sync {
    type IamStore: IamStore;
    type StsStore: StsStore;
    type SsoAdminStore: SsoAdminStore;

    async fn iam_store(&mut self) -> Result<&mut Self::IamStore>;
    async fn sts_store(&mut self) -> Result<&mut Self::StsStore>;
    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore>;
}
