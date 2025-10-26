use crate::error::Result;
use crate::iam::{IamClient, Policy};
use crate::provider::ResourceType;
use crate::store::{IamStore, Store};
use crate::types::{AmiResponse, PaginationParams};
use serde::{Deserialize, Serialize};

/// Request to create an IAM managed policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyRequest {
    /// The friendly name of the policy
    pub policy_name: String,
    /// The policy document in JSON format
    pub policy_document: String,
    /// The path for the policy (defaults to "/")
    pub path: Option<String>,
    /// A friendly description of the policy
    pub description: Option<String>,
    /// A list of tags to attach to the policy
    pub tags: Option<Vec<crate::types::Tag>>,
}

/// Request to update an IAM managed policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicyRequest {
    /// The ARN of the policy to update
    pub policy_arn: String,
    /// A new description for the policy
    pub description: Option<String>,
    /// The new default version ID (optional)
    pub default_version_id: Option<String>,
}

/// Request to list IAM managed policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPoliciesRequest {
    /// The scope to use for filtering ("All", "AWS", or "Local")
    pub scope: Option<String>,
    /// Only list policies attached to the specified user, group, or role
    pub only_attached: Option<bool>,
    /// The path prefix for filtering policies
    pub path_prefix: Option<String>,
    /// Pagination parameters
    pub pagination: Option<PaginationParams>,
}

/// Response for list policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPoliciesResponse {
    /// The list of policies
    pub policies: Vec<Policy>,
    /// Whether there are more results
    pub is_truncated: bool,
    /// The marker for the next page
    pub marker: Option<String>,
}

impl<S: Store> IamClient<S> {
    /// Create a managed IAM policy
    ///
    /// # Arguments
    ///
    /// * `request` - The create policy request
    ///
    /// # Returns
    ///
    /// Returns the created policy
    ///
    /// # Errors
    ///
    /// * `InvalidParameter` - If the policy document is not valid JSON
    /// * `InvalidParameter` - If the policy name is invalid
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreatePolicyRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let policy_document = r#"{
    ///     "Version": "2012-10-17",
    ///     "Statement": [{
    ///         "Effect": "Allow",
    ///         "Action": "s3:GetObject",
    ///         "Resource": "arn:aws:s3:::my-bucket/*"
    ///     }]
    /// }"#;
    ///
    /// let request = CreatePolicyRequest {
    ///     policy_name: "S3ReadPolicy".to_string(),
    ///     policy_document: policy_document.to_string(),
    ///     path: Some("/".to_string()),
    ///     description: Some("Allows reading from S3 bucket".to_string()),
    ///     tags: None,
    /// };
    ///
    /// let response = client.create_policy(request).await?;
    /// let policy = response.data.unwrap();
    /// assert_eq!(policy.policy_name, "S3ReadPolicy");
    /// assert_eq!(policy.default_version_id, "v1");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_policy(
        &mut self,
        request: CreatePolicyRequest,
    ) -> Result<AmiResponse<Policy>> {
        // Validate policy document is valid JSON
        Self::validate_policy_document(&request.policy_document)?;

        let store = self.iam_store().await?;
        let account_id = store.account_id();
        let provider = store.cloud_provider();

        // Use provider for policy ID and ARN generation
        let policy_id = provider.generate_resource_id(ResourceType::Policy);
        let path = request.path.unwrap_or_else(|| "/".to_string());
        let arn = provider.generate_resource_identifier(
            ResourceType::Policy,
            account_id,
            &path,
            &request.policy_name,
        );

        let policy = Policy {
            policy_name: request.policy_name.clone(),
            policy_id: policy_id.clone(),
            arn: arn.clone(),
            path,
            default_version_id: "v1".to_string(),
            policy_document: request.policy_document,
            attachment_count: 0,
            permissions_boundary_usage_count: 0,
            is_attachable: true,
            description: request.description,
            create_date: chrono::Utc::now(),
            update_date: chrono::Utc::now(),
            tags: request.tags.unwrap_or_default(),
        };

        let created_policy = store.create_policy(policy).await?;

        Ok(AmiResponse::success(created_policy))
    }

    /// Get an IAM managed policy
    ///
    /// # Arguments
    ///
    /// * `policy_arn` - The ARN of the policy to retrieve
    ///
    /// # Returns
    ///
    /// Returns the policy if found
    ///
    /// # Errors
    ///
    /// * `ResourceNotFound` - If the policy doesn't exist
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreatePolicyRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create a policy first
    /// let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
    /// let request = CreatePolicyRequest {
    ///     policy_name: "TestPolicy".to_string(),
    ///     policy_document: policy_document.to_string(),
    ///     path: None,
    ///     description: None,
    ///     tags: None,
    /// };
    /// let create_response = client.create_policy(request).await?;
    /// let created_policy = create_response.data.unwrap();
    ///
    /// // Get the policy
    /// let response = client.get_policy(created_policy.arn.clone()).await?;
    /// let policy = response.data.unwrap();
    /// assert_eq!(policy.policy_name, "TestPolicy");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_policy(&mut self, policy_arn: String) -> Result<AmiResponse<Policy>> {
        let store = self.iam_store().await?;
        match store.get_policy(&policy_arn).await? {
            Some(policy) => Ok(AmiResponse::success(policy)),
            None => Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Policy: {}", policy_arn),
            }),
        }
    }

    /// Update an IAM managed policy
    ///
    /// # Arguments
    ///
    /// * `request` - The update policy request
    ///
    /// # Returns
    ///
    /// Returns the updated policy
    ///
    /// # Errors
    ///
    /// * `ResourceNotFound` - If the policy doesn't exist
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreatePolicyRequest, UpdatePolicyRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create a policy
    /// let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
    /// let create_request = CreatePolicyRequest {
    ///     policy_name: "TestPolicy".to_string(),
    ///     policy_document: policy_document.to_string(),
    ///     path: None,
    ///     description: Some("Original description".to_string()),
    ///     tags: None,
    /// };
    /// let create_response = client.create_policy(create_request).await?;
    /// let created_policy = create_response.data.unwrap();
    ///
    /// // Update the policy
    /// let update_request = UpdatePolicyRequest {
    ///     policy_arn: created_policy.arn.clone(),
    ///     description: Some("Updated description".to_string()),
    ///     default_version_id: Some("v2".to_string()),
    /// };
    ///
    /// let response = client.update_policy(update_request).await?;
    /// let policy = response.data.unwrap();
    /// assert_eq!(policy.description, Some("Updated description".to_string()));
    /// assert_eq!(policy.default_version_id, "v2");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_policy(
        &mut self,
        request: UpdatePolicyRequest,
    ) -> Result<AmiResponse<Policy>> {
        let store = self.iam_store().await?;

        // Get existing policy
        let mut policy = match store.get_policy(&request.policy_arn).await? {
            Some(p) => p,
            None => {
                return Err(crate::error::AmiError::ResourceNotFound {
                    resource: format!("Policy: {}", request.policy_arn),
                });
            }
        };

        // Update fields
        if let Some(description) = request.description {
            policy.description = Some(description);
        }
        if let Some(version_id) = request.default_version_id {
            policy.default_version_id = version_id;
        }
        policy.update_date = chrono::Utc::now();

        let updated_policy = store.update_policy(policy).await?;

        Ok(AmiResponse::success(updated_policy))
    }

    /// Delete an IAM managed policy
    ///
    /// # Arguments
    ///
    /// * `policy_arn` - The ARN of the policy to delete
    ///
    /// # Returns
    ///
    /// Returns success if the policy was deleted
    ///
    /// # Errors
    ///
    /// * `ResourceNotFound` - If the policy doesn't exist
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreatePolicyRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create a policy
    /// let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
    /// let request = CreatePolicyRequest {
    ///     policy_name: "TestPolicy".to_string(),
    ///     policy_document: policy_document.to_string(),
    ///     path: None,
    ///     description: None,
    ///     tags: None,
    /// };
    /// let create_response = client.create_policy(request).await?;
    /// let created_policy = create_response.data.unwrap();
    ///
    /// // Delete the policy
    /// let response = client.delete_policy(created_policy.arn.clone()).await?;
    /// assert!(response.success);
    ///
    /// // Verify it's deleted
    /// let result = client.get_policy(created_policy.arn).await;
    /// assert!(result.is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_policy(&mut self, policy_arn: String) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Check if policy exists
        if store.get_policy(&policy_arn).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Policy: {}", policy_arn),
            });
        }

        store.delete_policy(&policy_arn).await?;

        Ok(AmiResponse::success(()))
    }

    /// List IAM managed policies
    ///
    /// # Arguments
    ///
    /// * `request` - The list policies request with optional filters
    ///
    /// # Returns
    ///
    /// Returns a list of policies with pagination info
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreatePolicyRequest, ListPoliciesRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create a few policies
    /// for i in 1..=3 {
    ///     let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
    ///     let request = CreatePolicyRequest {
    ///         policy_name: format!("TestPolicy{}", i),
    ///         policy_document: policy_document.to_string(),
    ///         path: Some("/test/".to_string()),
    ///         description: None,
    ///         tags: None,
    ///     };
    ///     client.create_policy(request).await?;
    /// }
    ///
    /// // List policies
    /// let list_request = ListPoliciesRequest {
    ///     scope: None,
    ///     only_attached: None,
    ///     path_prefix: Some("/test/".to_string()),
    ///     pagination: None,
    /// };
    ///
    /// let response = client.list_policies(list_request).await?;
    /// let result = response.data.unwrap();
    /// assert_eq!(result.policies.len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_policies(
        &mut self,
        request: ListPoliciesRequest,
    ) -> Result<AmiResponse<ListPoliciesResponse>> {
        let store = self.iam_store().await?;

        let (mut policies, is_truncated, marker) = store
            .list_policies(request.scope.as_deref(), request.pagination.as_ref())
            .await?;

        // Filter by path prefix if specified
        if let Some(path_prefix) = request.path_prefix {
            policies.retain(|p| p.path.starts_with(&path_prefix));
        }

        // Filter by attachment status if specified
        if let Some(true) = request.only_attached {
            policies.retain(|p| p.attachment_count > 0);
        }

        let response = ListPoliciesResponse {
            policies,
            is_truncated,
            marker,
        };

        Ok(AmiResponse::success(response))
    }

    /// Validate a policy document is valid JSON
    ///
    /// Basic validation to ensure the policy document is properly formatted JSON
    #[allow(clippy::result_large_err)]
    fn validate_policy_document(document: &str) -> Result<()> {
        // Validate it's valid JSON
        if serde_json::from_str::<serde_json::Value>(document).is_err() {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Policy document is not valid JSON".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryStore;

    #[tokio::test]
    async fn test_create_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let policy_document = r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "s3:GetObject",
                "Resource": "*"
            }]
        }"#;

        let request = CreatePolicyRequest {
            policy_name: "S3ReadPolicy".to_string(),
            policy_document: policy_document.to_string(),
            path: Some("/".to_string()),
            description: Some("Test policy".to_string()),
            tags: None,
        };

        let response = client.create_policy(request).await.unwrap();
        let policy = response.data.unwrap();

        assert_eq!(policy.policy_name, "S3ReadPolicy");
        assert!(policy.policy_id.starts_with("ANPA"));
        assert_eq!(policy.default_version_id, "v1");
        assert_eq!(policy.attachment_count, 0);
        assert_eq!(policy.description, Some("Test policy".to_string()));
    }

    #[tokio::test]
    async fn test_create_policy_invalid_document() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let request = CreatePolicyRequest {
            policy_name: "InvalidPolicy".to_string(),
            policy_document: "not a valid json".to_string(),
            path: None,
            description: None,
            tags: None,
        };

        let result = client.create_policy(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create a policy
        let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
        let request = CreatePolicyRequest {
            policy_name: "TestPolicy".to_string(),
            policy_document: policy_document.to_string(),
            path: None,
            description: None,
            tags: None,
        };
        let create_response = client.create_policy(request).await.unwrap();
        let created_policy = create_response.data.unwrap();

        // Get the policy
        let response = client.get_policy(created_policy.arn.clone()).await.unwrap();
        let policy = response.data.unwrap();

        assert_eq!(policy.policy_name, "TestPolicy");
        assert_eq!(policy.arn, created_policy.arn);
    }

    #[tokio::test]
    async fn test_get_nonexistent_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let result = client
            .get_policy("arn:aws:iam::123456789012:policy/NonExistent".to_string())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create a policy
        let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
        let create_request = CreatePolicyRequest {
            policy_name: "TestPolicy".to_string(),
            policy_document: policy_document.to_string(),
            path: None,
            description: Some("Original".to_string()),
            tags: None,
        };
        let create_response = client.create_policy(create_request).await.unwrap();
        let created_policy = create_response.data.unwrap();

        // Update the policy
        let update_request = UpdatePolicyRequest {
            policy_arn: created_policy.arn.clone(),
            description: Some("Updated".to_string()),
            default_version_id: Some("v2".to_string()),
        };

        let response = client.update_policy(update_request).await.unwrap();
        let policy = response.data.unwrap();

        assert_eq!(policy.description, Some("Updated".to_string()));
        assert_eq!(policy.default_version_id, "v2");
    }

    #[tokio::test]
    async fn test_update_nonexistent_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let request = UpdatePolicyRequest {
            policy_arn: "arn:aws:iam::123456789012:policy/NonExistent".to_string(),
            description: Some("Updated".to_string()),
            default_version_id: None,
        };

        let result = client.update_policy(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create a policy
        let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
        let request = CreatePolicyRequest {
            policy_name: "TestPolicy".to_string(),
            policy_document: policy_document.to_string(),
            path: None,
            description: None,
            tags: None,
        };
        let create_response = client.create_policy(request).await.unwrap();
        let created_policy = create_response.data.unwrap();

        // Delete the policy
        let response = client
            .delete_policy(created_policy.arn.clone())
            .await
            .unwrap();
        assert!(response.success);

        // Verify it's deleted
        let result = client.get_policy(created_policy.arn).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let result = client
            .delete_policy("arn:aws:iam::123456789012:policy/NonExistent".to_string())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_policies() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create a few policies
        let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;
        for i in 1..=3 {
            let request = CreatePolicyRequest {
                policy_name: format!("TestPolicy{}", i),
                policy_document: policy_document.to_string(),
                path: Some("/test/".to_string()),
                description: None,
                tags: None,
            };
            client.create_policy(request).await.unwrap();
        }

        // List all policies
        let list_request = ListPoliciesRequest {
            scope: None,
            only_attached: None,
            path_prefix: None,
            pagination: None,
        };

        let response = client.list_policies(list_request).await.unwrap();
        let result = response.data.unwrap();
        assert_eq!(result.policies.len(), 3);
    }

    #[tokio::test]
    async fn test_list_policies_with_path_filter() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create policies with different paths
        let policy_document = r#"{"Version": "2012-10-17", "Statement": []}"#;

        let request1 = CreatePolicyRequest {
            policy_name: "Policy1".to_string(),
            policy_document: policy_document.to_string(),
            path: Some("/test/".to_string()),
            description: None,
            tags: None,
        };
        client.create_policy(request1).await.unwrap();

        let request2 = CreatePolicyRequest {
            policy_name: "Policy2".to_string(),
            policy_document: policy_document.to_string(),
            path: Some("/prod/".to_string()),
            description: None,
            tags: None,
        };
        client.create_policy(request2).await.unwrap();

        // List only /test/ policies
        let list_request = ListPoliciesRequest {
            scope: None,
            only_attached: None,
            path_prefix: Some("/test/".to_string()),
            pagination: None,
        };

        let response = client.list_policies(list_request).await.unwrap();
        let result = response.data.unwrap();
        assert_eq!(result.policies.len(), 1);
        assert_eq!(result.policies[0].policy_name, "Policy1");
    }

    #[tokio::test]
    async fn test_policy_document_validation() {
        // Valid JSON
        assert!(IamClient::<InMemoryStore>::validate_policy_document(
            r#"{"Version": "2012-10-17"}"#
        )
        .is_ok());

        // Invalid JSON
        assert!(IamClient::<InMemoryStore>::validate_policy_document("not json").is_err());
    }
}
