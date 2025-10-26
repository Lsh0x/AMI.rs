//! SSO Admin Store Trait
//!
//! Defines the interface for SSO Admin data storage operations.

use crate::error::Result;
use crate::sso_admin::{
    AccountAssignment, Application, PermissionSet, SsoInstance, TrustedTokenIssuer,
};
use async_trait::async_trait;

/// Trait for SSO Admin data storage operations
#[async_trait]
pub trait SsoAdminStore: Send + Sync {
    // Permission Set operations
    async fn create_permission_set(
        &mut self,
        permission_set: PermissionSet,
    ) -> Result<PermissionSet>;
    async fn get_permission_set(&self, permission_set_arn: &str) -> Result<Option<PermissionSet>>;
    async fn update_permission_set(
        &mut self,
        permission_set: PermissionSet,
    ) -> Result<PermissionSet>;
    async fn delete_permission_set(&mut self, permission_set_arn: &str) -> Result<()>;
    async fn list_permission_sets(&self, instance_arn: &str) -> Result<Vec<PermissionSet>>;

    // Account Assignment operations
    async fn create_account_assignment(
        &mut self,
        assignment: AccountAssignment,
    ) -> Result<AccountAssignment>;
    async fn get_account_assignment(
        &self,
        assignment_id: &str,
    ) -> Result<Option<AccountAssignment>>;
    async fn delete_account_assignment(&mut self, assignment_id: &str) -> Result<()>;
    async fn list_account_assignments(
        &self,
        account_id: &str,
        permission_set_arn: &str,
    ) -> Result<Vec<AccountAssignment>>;

    // SSO Instance operations
    async fn create_instance(&mut self, instance: SsoInstance) -> Result<SsoInstance>;
    async fn get_instance(&self, instance_arn: &str) -> Result<Option<SsoInstance>>;
    async fn list_instances(&self) -> Result<Vec<SsoInstance>>;

    // Application operations
    async fn create_application(&mut self, application: Application) -> Result<Application>;
    async fn get_application(&self, application_arn: &str) -> Result<Option<Application>>;
    async fn list_applications(&self, instance_arn: &str) -> Result<Vec<Application>>;

    // Trusted Token Issuer operations
    async fn create_trusted_token_issuer(
        &mut self,
        issuer: TrustedTokenIssuer,
    ) -> Result<TrustedTokenIssuer>;
    async fn get_trusted_token_issuer(
        &self,
        issuer_arn: &str,
    ) -> Result<Option<TrustedTokenIssuer>>;
    async fn delete_trusted_token_issuer(&mut self, issuer_arn: &str) -> Result<()>;
    async fn list_trusted_token_issuers(
        &self,
        instance_arn: &str,
    ) -> Result<Vec<TrustedTokenIssuer>>;
}
