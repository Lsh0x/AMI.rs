//! Permission Set Store Trait

use crate::error::Result;
use crate::wami::sso_admin::PermissionSet;
use async_trait::async_trait;

/// Trait for permission set storage operations
#[async_trait]
pub trait PermissionSetStore: Send + Sync {
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
}
