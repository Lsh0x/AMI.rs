//! IAM Store Trait
//!
//! Defines the interface for IAM data storage operations.
//! This is a pure persistence layer - resources carry their own tenant/account/provider info.

use crate::error::Result;
use crate::iam::{AccessKey, Group, LoginProfile, MfaDevice, Policy, Role, User};
use crate::types::{PaginationParams, Tag};
use async_trait::async_trait;

/// Trait for IAM data storage operations
#[async_trait]
pub trait IamStore: Send + Sync {
    // User operations
    async fn create_user(&mut self, user: User) -> Result<User>;
    async fn get_user(&self, user_name: &str) -> Result<Option<User>>;
    async fn update_user(&mut self, user: User) -> Result<User>;
    async fn delete_user(&mut self, user_name: &str) -> Result<()>;
    async fn list_users(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<User>, bool, Option<String>)>;
    async fn tag_user(&mut self, user_name: &str, tags: Vec<Tag>) -> Result<()>;
    async fn untag_user(&mut self, user_name: &str, tag_keys: Vec<String>) -> Result<()>;
    async fn list_user_tags(&self, user_name: &str) -> Result<Vec<Tag>>;

    // Access key operations
    async fn create_access_key(&mut self, access_key: AccessKey) -> Result<AccessKey>;
    async fn get_access_key(&self, access_key_id: &str) -> Result<Option<AccessKey>>;
    async fn update_access_key(&mut self, access_key: AccessKey) -> Result<AccessKey>;
    async fn delete_access_key(&mut self, access_key_id: &str) -> Result<()>;
    async fn list_access_keys(
        &self,
        user_name: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<AccessKey>, bool, Option<String>)>;

    // Group operations
    async fn create_group(&mut self, group: Group) -> Result<Group>;
    async fn get_group(&self, group_name: &str) -> Result<Option<Group>>;
    async fn update_group(&mut self, group: Group) -> Result<Group>;
    async fn delete_group(&mut self, group_name: &str) -> Result<()>;
    async fn list_groups(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Group>, bool, Option<String>)>;
    async fn list_groups_for_user(&self, user_name: &str) -> Result<Vec<Group>>;
    async fn add_user_to_group(&mut self, group_name: &str, user_name: &str) -> Result<()>;
    async fn remove_user_from_group(&mut self, group_name: &str, user_name: &str) -> Result<()>;
    async fn tag_group(&mut self, group_name: &str, tags: Vec<Tag>) -> Result<()>;
    async fn untag_group(&mut self, group_name: &str, tag_keys: Vec<String>) -> Result<()>;
    async fn list_group_tags(&self, group_name: &str) -> Result<Vec<Tag>>;

    // Role operations
    async fn create_role(&mut self, role: Role) -> Result<Role>;
    async fn get_role(&self, role_name: &str) -> Result<Option<Role>>;
    async fn update_role(&mut self, role: Role) -> Result<Role>;
    async fn delete_role(&mut self, role_name: &str) -> Result<()>;
    async fn list_roles(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Role>, bool, Option<String>)>;
    async fn tag_role(&mut self, role_name: &str, tags: Vec<Tag>) -> Result<()>;
    async fn untag_role(&mut self, role_name: &str, tag_keys: Vec<String>) -> Result<()>;
    async fn list_role_tags(&self, role_name: &str) -> Result<Vec<Tag>>;

    // Policy operations
    async fn create_policy(&mut self, policy: Policy) -> Result<Policy>;
    async fn get_policy(&self, policy_arn: &str) -> Result<Option<Policy>>;
    async fn update_policy(&mut self, policy: Policy) -> Result<Policy>;
    async fn delete_policy(&mut self, policy_arn: &str) -> Result<()>;
    async fn list_policies(
        &self,
        scope: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Policy>, bool, Option<String>)>;
    async fn tag_policy(&mut self, policy_arn: &str, tags: Vec<Tag>) -> Result<()>;
    async fn untag_policy(&mut self, policy_arn: &str, tag_keys: Vec<String>) -> Result<()>;
    async fn list_policy_tags(&self, policy_arn: &str) -> Result<Vec<Tag>>;

    // MFA device operations
    async fn create_mfa_device(&mut self, mfa_device: MfaDevice) -> Result<MfaDevice>;
    async fn get_mfa_device(&self, serial_number: &str) -> Result<Option<MfaDevice>>;
    async fn delete_mfa_device(&mut self, serial_number: &str) -> Result<()>;
    async fn list_mfa_devices(&self, user_name: &str) -> Result<Vec<MfaDevice>>;

    // Login profile (password) operations
    async fn create_login_profile(&mut self, profile: LoginProfile) -> Result<LoginProfile>;
    async fn get_login_profile(&self, user_name: &str) -> Result<Option<LoginProfile>>;
    async fn update_login_profile(&mut self, profile: LoginProfile) -> Result<LoginProfile>;
    async fn delete_login_profile(&mut self, user_name: &str) -> Result<()>;

    // Credential report operations
    async fn store_credential_report(
        &mut self,
        report: crate::iam::report::CredentialReport,
    ) -> Result<()>;
    async fn get_credential_report(&self) -> Result<Option<crate::iam::report::CredentialReport>>;

    // Server certificate operations
    async fn create_server_certificate(
        &mut self,
        certificate: crate::iam::ServerCertificate,
    ) -> Result<crate::iam::ServerCertificate>;
    async fn get_server_certificate(
        &self,
        certificate_name: &str,
    ) -> Result<Option<crate::iam::ServerCertificate>>;
    async fn update_server_certificate(
        &mut self,
        certificate: crate::iam::ServerCertificate,
    ) -> Result<crate::iam::ServerCertificate>;
    async fn delete_server_certificate(&mut self, certificate_name: &str) -> Result<()>;
    async fn list_server_certificates(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<crate::iam::ServerCertificate>, bool, Option<String>)>;

    // Service-specific credential operations
    async fn create_service_specific_credential(
        &mut self,
        credential: crate::iam::service_credential::ServiceSpecificCredential,
    ) -> Result<crate::iam::service_credential::ServiceSpecificCredential>;
    async fn get_service_specific_credential(
        &self,
        credential_id: &str,
    ) -> Result<Option<crate::iam::service_credential::ServiceSpecificCredential>>;
    async fn update_service_specific_credential(
        &mut self,
        credential: crate::iam::service_credential::ServiceSpecificCredential,
    ) -> Result<crate::iam::service_credential::ServiceSpecificCredential>;
    async fn delete_service_specific_credential(&mut self, credential_id: &str) -> Result<()>;
    async fn list_service_specific_credentials(
        &self,
        user_name: Option<&str>,
        service_name: Option<&str>,
    ) -> Result<Vec<crate::iam::service_credential::ServiceSpecificCredential>>;

    // Service-linked role deletion task operations
    async fn create_service_linked_role_deletion_task(
        &mut self,
        task: crate::iam::service_linked_role::DeletionTaskInfo,
    ) -> Result<crate::iam::service_linked_role::DeletionTaskInfo>;
    async fn get_service_linked_role_deletion_task(
        &self,
        task_id: &str,
    ) -> Result<crate::iam::service_linked_role::DeletionTaskInfo>;
    async fn update_service_linked_role_deletion_task(
        &mut self,
        task: crate::iam::service_linked_role::DeletionTaskInfo,
    ) -> Result<crate::iam::service_linked_role::DeletionTaskInfo>;

    // Signing certificate operations
    async fn create_signing_certificate(
        &mut self,
        certificate: crate::iam::signing_certificate::SigningCertificate,
    ) -> Result<crate::iam::signing_certificate::SigningCertificate>;
    async fn get_signing_certificate(
        &self,
        certificate_id: &str,
    ) -> Result<Option<crate::iam::signing_certificate::SigningCertificate>>;
    async fn update_signing_certificate(
        &mut self,
        certificate: crate::iam::signing_certificate::SigningCertificate,
    ) -> Result<crate::iam::signing_certificate::SigningCertificate>;
    async fn delete_signing_certificate(&mut self, certificate_id: &str) -> Result<()>;
    async fn list_signing_certificates(
        &self,
        user_name: Option<&str>,
    ) -> Result<Vec<crate::iam::signing_certificate::SigningCertificate>>;
}
