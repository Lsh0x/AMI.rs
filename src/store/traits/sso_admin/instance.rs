//! SSO Instance Store Trait

use crate::error::Result;
use crate::wami::sso_admin::SsoInstance;
use async_trait::async_trait;

/// Trait for SSO instance storage operations
#[async_trait]
pub trait SsoInstanceStore: Send + Sync {
    async fn create_instance(&mut self, instance: SsoInstance) -> Result<SsoInstance>;

    async fn get_instance(&self, instance_arn: &str) -> Result<Option<SsoInstance>>;

    async fn list_instances(&self) -> Result<Vec<SsoInstance>>;
}
