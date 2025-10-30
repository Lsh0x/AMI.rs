//! Service-Linked Role Service
//!
//! Orchestrates service-linked role management operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::{RoleStore, ServiceLinkedRoleStore};
use crate::wami::identity::role::builder as role_builder;
use crate::wami::identity::service_linked_role::{
    operations as slr_ops, CreateServiceLinkedRoleRequest, DeletionTaskInfo,
};
use crate::wami::identity::Role;
use std::sync::{Arc, RwLock};

/// Service for managing service-linked roles
///
/// Service-linked roles are predefined AWS roles that are linked to specific AWS services.
pub struct ServiceLinkedRoleService<S> {
    store: Arc<RwLock<S>>,
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: RoleStore + ServiceLinkedRoleStore> ServiceLinkedRoleService<S> {
    /// Create a new ServiceLinkedRoleService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>, account_id: String) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
            account_id,
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
            account_id: self.account_id.clone(),
        }
    }

    /// Create a service-linked role
    pub async fn create_service_linked_role(
        &self,
        request: CreateServiceLinkedRoleRequest,
    ) -> Result<Role> {
        // Validate service name
        slr_ops::service_linked_role_operations::validate_service_name(&request.aws_service_name)?;

        // Validate custom suffix if provided
        if let Some(ref suffix) = request.custom_suffix {
            slr_ops::service_linked_role_operations::validate_custom_suffix(suffix)?;
        }

        // Generate role name
        let role_name = slr_ops::service_linked_role_operations::generate_role_name(
            &request.aws_service_name,
            request.custom_suffix.as_deref(),
        );

        // Service-linked roles use a fixed path
        let path = "/aws-service-role/".to_string() + &request.aws_service_name + "/";

        // Build assume role policy document for service-linked role
        let assume_role_policy = format!(
            r#"{{"Version":"2012-10-17","Statement":[{{"Effect":"Allow","Principal":{{"Service":"{}"}},"Action":"sts:AssumeRole"}}]}}"#,
            request.aws_service_name
        );

        // Use wami role builder to create the role
        let role = role_builder::build_role(
            role_name,
            assume_role_policy,
            Some(path),
            request.description,
            None, // max_session_duration
            &*self.provider,
            &self.account_id,
        );

        // Store it (service-linked roles are stored as regular roles)
        self.store.write().unwrap().create_role(role).await
    }

    /// Get the status of a service-linked role deletion task
    pub async fn get_service_linked_role_deletion_task(
        &self,
        deletion_task_id: &str,
    ) -> Result<Option<DeletionTaskInfo>> {
        self.store
            .read()
            .unwrap()
            .get_service_linked_role_deletion_task(deletion_task_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> ServiceLinkedRoleService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        ServiceLinkedRoleService::new(store, "123456789012".to_string())
    }

    #[tokio::test]
    async fn test_create_service_linked_role() {
        let service = setup_service();

        let request = CreateServiceLinkedRoleRequest {
            aws_service_name: "elasticbeanstalk.amazonaws.com".to_string(),
            description: Some("Service-linked role for Elastic Beanstalk".to_string()),
            custom_suffix: None,
        };

        let role = service.create_service_linked_role(request).await.unwrap();
        assert!(role.role_name.contains("AWSServiceRoleForElasticbeanstalk"));
        assert_eq!(
            role.path,
            "/aws-service-role/elasticbeanstalk.amazonaws.com/"
        );
    }

    #[tokio::test]
    async fn test_create_service_linked_role_with_custom_suffix() {
        let service = setup_service();

        let request = CreateServiceLinkedRoleRequest {
            aws_service_name: "autoscaling.amazonaws.com".to_string(),
            description: None,
            custom_suffix: Some("MyApp".to_string()),
        };

        let role = service.create_service_linked_role(request).await.unwrap();
        assert!(role.role_name.contains("MyApp"));
    }

    #[tokio::test]
    async fn test_create_service_linked_role_invalid_service() {
        let service = setup_service();

        let request = CreateServiceLinkedRoleRequest {
            aws_service_name: "invalid-service".to_string(),
            description: None,
            custom_suffix: None,
        };

        let result = service.create_service_linked_role(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_deletion_task() {
        let service = setup_service();

        // Create a role first (in real scenario, deletion would create a task)
        let request = CreateServiceLinkedRoleRequest {
            aws_service_name: "elasticbeanstalk.amazonaws.com".to_string(),
            description: None,
            custom_suffix: None,
        };
        service.create_service_linked_role(request).await.unwrap();

        // Try to get a nonexistent deletion task
        let task = service
            .get_service_linked_role_deletion_task("task-123")
            .await
            .unwrap();
        assert!(task.is_none());
    }
}
