//! Policy Service
//!
//! Orchestrates policy management operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::PolicyStore;
use crate::wami::policies::policy::{
    builder as policy_builder, CreatePolicyRequest, ListPoliciesRequest, Policy,
    UpdatePolicyRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM policies
///
/// Provides high-level operations for policy management.
pub struct PolicyService<S> {
    store: Arc<RwLock<S>>,
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: PolicyStore> PolicyService<S> {
    /// Create a new PolicyService with default AWS provider
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

    /// Create a new policy
    pub async fn create_policy(&self, request: CreatePolicyRequest) -> Result<Policy> {
        // Use wami builder to create policy (includes tags)
        let policy = policy_builder::build_policy(
            request.policy_name,
            request.policy_document,
            request.path,
            request.description,
            request.tags,
            &*self.provider,
            &self.account_id,
        );

        // Store it
        self.store.write().unwrap().create_policy(policy).await
    }

    /// Get a policy by ARN
    pub async fn get_policy(&self, policy_arn: &str) -> Result<Option<Policy>> {
        self.store.read().unwrap().get_policy(policy_arn).await
    }

    /// Update a policy
    pub async fn update_policy(&self, request: UpdatePolicyRequest) -> Result<Policy> {
        // Get existing policy
        let policy = self
            .store
            .read()
            .unwrap()
            .get_policy(&request.policy_arn)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("Policy: {}", request.policy_arn),
            })?;

        // Apply updates using builder function
        let updated_policy =
            policy_builder::update_policy(policy, request.description, request.default_version_id);

        // Store updated policy
        self.store
            .write()
            .unwrap()
            .update_policy(updated_policy)
            .await
    }

    /// Delete a policy
    pub async fn delete_policy(&self, policy_arn: &str) -> Result<()> {
        self.store.write().unwrap().delete_policy(policy_arn).await
    }

    /// List policies with optional filtering
    pub async fn list_policies(
        &self,
        request: ListPoliciesRequest,
    ) -> Result<(Vec<Policy>, bool, Option<String>)> {
        self.store
            .read()
            .unwrap()
            .list_policies(request.scope.as_deref(), request.pagination.as_ref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> PolicyService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        PolicyService::new(store, "123456789012".to_string())
    }

    #[tokio::test]
    async fn test_create_and_get_policy() {
        let service = setup_service();

        let policy_doc = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":"s3:*","Resource":"*"}]}"#;
        let request = CreatePolicyRequest {
            policy_name: "S3FullAccess".to_string(),
            policy_document: policy_doc.to_string(),
            path: Some("/service/".to_string()),
            description: Some("Full S3 access policy".to_string()),
            tags: None,
        };

        let policy = service.create_policy(request).await.unwrap();
        assert_eq!(policy.policy_name, "S3FullAccess");
        assert_eq!(policy.path, "/service/");
        assert_eq!(
            policy.description,
            Some("Full S3 access policy".to_string())
        );

        let retrieved = service.get_policy(&policy.arn).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().policy_name, "S3FullAccess");
    }

    #[tokio::test]
    async fn test_update_policy() {
        let service = setup_service();

        // Create policy
        let policy_doc = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":"ec2:*","Resource":"*"}]}"#;
        let create_request = CreatePolicyRequest {
            policy_name: "EC2FullAccess".to_string(),
            policy_document: policy_doc.to_string(),
            path: Some("/".to_string()),
            description: Some("Original description".to_string()),
            tags: None,
        };
        let policy = service.create_policy(create_request).await.unwrap();

        // Update policy
        let update_request = UpdatePolicyRequest {
            policy_arn: policy.arn.clone(),
            description: Some("Updated description".to_string()),
            default_version_id: Some("v2".to_string()),
        };
        let updated = service.update_policy(update_request).await.unwrap();
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.default_version_id, "v2");
    }

    #[tokio::test]
    async fn test_delete_policy() {
        let service = setup_service();

        let policy_doc = r#"{"Version":"2012-10-17","Statement":[]}"#;
        let request = CreatePolicyRequest {
            policy_name: "TempPolicy".to_string(),
            policy_document: policy_doc.to_string(),
            path: None,
            description: None,
            tags: None,
        };
        let policy = service.create_policy(request).await.unwrap();

        service.delete_policy(&policy.arn).await.unwrap();

        let retrieved = service.get_policy(&policy.arn).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_policies() {
        let service = setup_service();

        // Create multiple policies
        for i in 0..3 {
            let policy_doc = r#"{"Version":"2012-10-17","Statement":[]}"#;
            let request = CreatePolicyRequest {
                policy_name: format!("Policy{}", i),
                policy_document: policy_doc.to_string(),
                path: Some("/test/".to_string()),
                description: None,
                tags: None,
            };
            service.create_policy(request).await.unwrap();
        }

        let list_request = ListPoliciesRequest {
            scope: None,
            only_attached: None,
            path_prefix: Some("/test/".to_string()),
            pagination: None,
        };
        let (policies, _, _) = service.list_policies(list_request).await.unwrap();
        assert_eq!(policies.len(), 3);
    }
}
