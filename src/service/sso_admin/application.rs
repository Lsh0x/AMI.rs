//! Application Service
//!
//! Orchestrates application operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::ApplicationStore;
use crate::wami::sso_admin::application::Application;
use std::sync::{Arc, RwLock};

/// Service for managing applications
pub struct ApplicationService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
}

impl<S: ApplicationStore> ApplicationService<S> {
    /// Create a new ApplicationService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
        }
    }

    /// Create a new application
    pub async fn create_application(&self, application: Application) -> Result<Application> {
        self.store
            .write()
            .unwrap()
            .create_application(application)
            .await
    }

    /// Get an application by ARN
    pub async fn get_application(&self, application_arn: &str) -> Result<Option<Application>> {
        self.store
            .read()
            .unwrap()
            .get_application(application_arn)
            .await
    }

    /// List applications for an instance
    pub async fn list_applications(&self, instance_arn: &str) -> Result<Vec<Application>> {
        self.store
            .read()
            .unwrap()
            .list_applications(instance_arn)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use chrono::Utc;

    fn setup_service() -> ApplicationService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        ApplicationService::new(store)
    }

    fn create_test_application(name: &str, instance_arn: &str) -> Application {
        Application {
            application_arn: format!("arn:aws:sso:::application/{}/app-{}", instance_arn, name),
            name: name.to_string(),
            description: Some(format!("Test app {}", name)),
            instance_arn: instance_arn.to_string(),
            application_provider_arn: None,
            status: "ACTIVE".to_string(),
            created_date: Utc::now(),
            portal_url: Some(format!("https://{}.example.com", name)),
            wami_arn: format!(
                "arn:wami:sso-admin:root:wami:123456789012:application/app-{}",
                name
            )
            .parse()
            .unwrap(),
            providers: vec![],
        }
    }

    #[tokio::test]
    async fn test_create_and_get_application() {
        let service = setup_service();
        let application = create_test_application("webapp", "instance-1");

        let created = service
            .create_application(application.clone())
            .await
            .unwrap();
        assert_eq!(created.name, "webapp");

        let retrieved = service
            .get_application(&application.application_arn)
            .await
            .unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_list_applications() {
        let service = setup_service();
        let instance_arn = "instance-1";

        service
            .create_application(create_test_application("app1", instance_arn))
            .await
            .unwrap();
        service
            .create_application(create_test_application("app2", instance_arn))
            .await
            .unwrap();

        let applications = service.list_applications(instance_arn).await.unwrap();
        assert_eq!(applications.len(), 2);
    }
}
