//! Application Store Trait

use crate::error::Result;
use crate::wami::sso_admin::Application;
use async_trait::async_trait;

/// Trait for SSO application storage operations
#[async_trait]
pub trait ApplicationStore: Send + Sync {
    async fn create_application(&mut self, application: Application) -> Result<Application>;

    async fn get_application(&self, application_arn: &str) -> Result<Option<Application>>;

    async fn list_applications(&self, instance_arn: &str) -> Result<Vec<Application>>;
}
