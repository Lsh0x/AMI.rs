//! Application Store Implementation for InMemorySsoAdminStore

use crate::error::Result;
use crate::store::memory::sso_admin::InMemorySsoAdminStore;
use crate::store::traits::ApplicationStore;
use crate::wami::sso_admin::Application;
use async_trait::async_trait;

#[async_trait]
impl ApplicationStore for InMemorySsoAdminStore {
    async fn create_application(&mut self, application: Application) -> Result<Application> {
        self.applications
            .insert(application.application_arn.clone(), application.clone());
        Ok(application)
    }

    async fn get_application(&self, application_arn: &str) -> Result<Option<Application>> {
        Ok(self.applications.get(application_arn).cloned())
    }

    async fn list_applications(&self, _instance_arn: &str) -> Result<Vec<Application>> {
        Ok(self.applications.values().cloned().collect())
    }
}
