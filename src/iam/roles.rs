use crate::error::Result;
use crate::iam::Role;
use crate::provider::ResourceType;
use crate::store::{IamStore, Store};
use crate::types::{AmiResponse, PaginationParams, Tag};
use serde::{Deserialize, Serialize};

/// Request parameters for creating a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    /// The name of the role
    pub role_name: String,
    /// The trust relationship policy document
    pub assume_role_policy_document: String,
    /// The path to the role
    pub path: Option<String>,
    /// A description of the role
    pub description: Option<String>,
    /// The maximum session duration in seconds (1h to 12h)
    pub max_session_duration: Option<i32>,
    /// The ARN of the policy used to set the permissions boundary
    pub permissions_boundary: Option<String>,
    /// Tags to attach to the role
    pub tags: Option<Vec<Tag>>,
}

/// Request parameters for updating a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    /// The name of the role to update
    pub role_name: String,
    /// New description
    pub description: Option<String>,
    /// New maximum session duration
    pub max_session_duration: Option<i32>,
}

/// Request parameters for listing roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolesRequest {
    /// Path prefix for filtering roles
    pub path_prefix: Option<String>,
    /// Pagination parameters
    pub pagination: Option<PaginationParams>,
}

/// Response for listing roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolesResponse {
    /// List of roles
    pub roles: Vec<Role>,
    /// Whether the results are truncated
    pub is_truncated: bool,
    /// Marker for pagination
    pub marker: Option<String>,
}

impl<S: Store> crate::iam::IamClient<S> {
    /// Create a new IAM role
    ///
    /// # Arguments
    ///
    /// * `request` - The request containing role name and trust policy
    ///
    /// # Returns
    ///
    /// Returns the newly created role.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The role name already exists
    /// * The assume role policy document is invalid JSON
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateRoleRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// let trust_policy = r#"{
    ///     "Version": "2012-10-17",
    ///     "Statement": [{
    ///         "Effect": "Allow",
    ///         "Principal": {"Service": "ec2.amazonaws.com"},
    ///         "Action": "sts:AssumeRole"
    ///     }]
    /// }"#;
    ///
    /// let request = CreateRoleRequest {
    ///     role_name: "EC2-S3-Access".to_string(),
    ///     assume_role_policy_document: trust_policy.to_string(),
    ///     path: Some("/service-role/".to_string()),
    ///     description: Some("Allows EC2 to access S3".to_string()),
    ///     max_session_duration: Some(3600),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    ///
    /// let response = iam_client.create_role(request).await?;
    /// let role = response.data.unwrap();
    /// println!("Created role: {}", role.arn);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_role(&mut self, request: CreateRoleRequest) -> Result<AmiResponse<Role>> {
        let store = self.iam_store().await?;
        let account_id = store.account_id();
        let provider = store.cloud_provider();

        // Validate max session duration using provider
        if let Some(duration) = request.max_session_duration {
            provider.validate_session_duration(duration)?;
        }

        // Validate that assume_role_policy_document is valid JSON
        if serde_json::from_str::<serde_json::Value>(&request.assume_role_policy_document).is_err()
        {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Assume role policy document must be valid JSON".to_string(),
            });
        }

        // Use provider for role ID and ARN generation
        let role_id = provider.generate_resource_id(ResourceType::Role);
        let path = request.path.unwrap_or_else(|| "/".to_string());
        let arn = provider.generate_resource_identifier(
            ResourceType::Role,
            account_id,
            &path,
            &request.role_name,
        );

        // Generate WAMI ARN for cross-provider identification
        let wami_arn =
            provider.generate_wami_arn(ResourceType::Role, account_id, &path, &request.role_name);

        let role = Role {
            role_name: request.role_name.clone(),
            role_id,
            arn,
            path,
            create_date: chrono::Utc::now(),
            assume_role_policy_document: request.assume_role_policy_document,
            description: request.description,
            max_session_duration: request.max_session_duration,
            permissions_boundary: request.permissions_boundary,
            tags: request.tags.unwrap_or_default(),
            wami_arn,
            providers: Vec::new(),
        };

        let created_role = store.create_role(role).await?;

        Ok(AmiResponse::success(created_role))
    }

    /// Update an IAM role
    ///
    /// # Arguments
    ///
    /// * `request` - The request containing role name and fields to update
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateRoleRequest, UpdateRoleRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Create a role first
    /// let create_request = CreateRoleRequest {
    ///     role_name: "MyRole".to_string(),
    ///     assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
    ///     path: None,
    ///     description: Some("Old description".to_string()),
    ///     max_session_duration: Some(3600),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_role(create_request).await?;
    ///
    /// // Update the role
    /// let update_request = UpdateRoleRequest {
    ///     role_name: "MyRole".to_string(),
    ///     description: Some("New description".to_string()),
    ///     max_session_duration: Some(7200),
    /// };
    /// let response = iam_client.update_role(update_request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_role(&mut self, request: UpdateRoleRequest) -> Result<AmiResponse<Role>> {
        let store = self.iam_store().await?;
        let provider = store.cloud_provider();

        // Validate max session duration if provided using provider
        if let Some(duration) = request.max_session_duration {
            provider.validate_session_duration(duration)?;
        }

        // Get existing role
        let mut role = match store.get_role(&request.role_name).await? {
            Some(role) => role,
            None => {
                return Err(crate::error::AmiError::ResourceNotFound {
                    resource: format!("Role: {}", request.role_name),
                })
            }
        };

        // Update fields
        if let Some(description) = request.description {
            role.description = Some(description);
        }
        if let Some(max_session_duration) = request.max_session_duration {
            role.max_session_duration = Some(max_session_duration);
        }

        let updated_role = store.update_role(role).await?;

        Ok(AmiResponse::success(updated_role))
    }

    /// Delete an IAM role
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to delete
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateRoleRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Create a role
    /// let request = CreateRoleRequest {
    ///     role_name: "ToDelete".to_string(),
    ///     assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
    ///     path: None,
    ///     description: None,
    ///     max_session_duration: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_role(request).await?;
    ///
    /// // Delete it
    /// iam_client.delete_role("ToDelete".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_role(&mut self, role_name: String) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Check if role exists before deleting
        if store.get_role(&role_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Role: {}", role_name),
            });
        }

        store.delete_role(&role_name).await?;
        Ok(AmiResponse::success(()))
    }

    /// Get information about a specific IAM role
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to retrieve
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateRoleRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Create a role
    /// let request = CreateRoleRequest {
    ///     role_name: "MyRole".to_string(),
    ///     assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
    ///     path: None,
    ///     description: Some("Test role".to_string()),
    ///     max_session_duration: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_role(request).await?;
    ///
    /// // Get the role
    /// let response = iam_client.get_role("MyRole".to_string()).await?;
    /// let role = response.data.unwrap();
    /// println!("Role ARN: {}", role.arn);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_role(&mut self, role_name: String) -> Result<AmiResponse<Role>> {
        let store = self.iam_store().await?;

        match store.get_role(&role_name).await? {
            Some(role) => Ok(AmiResponse::success(role)),
            None => Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Role: {}", role_name),
            }),
        }
    }

    /// List all IAM roles
    ///
    /// # Arguments
    ///
    /// * `request` - Optional request containing path prefix and pagination
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateRoleRequest, ListRolesRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Create some roles
    /// for i in 1..=3 {
    ///     let request = CreateRoleRequest {
    ///         role_name: format!("Role{}", i),
    ///         assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
    ///         path: Some("/service/".to_string()),
    ///         description: None,
    ///         max_session_duration: None,
    ///         permissions_boundary: None,
    ///         tags: None,
    ///     };
    ///     iam_client.create_role(request).await?;
    /// }
    ///
    /// // List them
    /// let list_request = ListRolesRequest {
    ///     path_prefix: Some("/service/".to_string()),
    ///     pagination: None,
    /// };
    /// let response = iam_client.list_roles(Some(list_request)).await?;
    /// let list_response = response.data.unwrap();
    /// println!("Found {} roles", list_response.roles.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_roles(
        &mut self,
        request: Option<ListRolesRequest>,
    ) -> Result<AmiResponse<ListRolesResponse>> {
        let store = self.iam_store().await?;

        let path_prefix = request.as_ref().and_then(|r| r.path_prefix.as_deref());
        let pagination = request.as_ref().and_then(|r| r.pagination.as_ref());

        let (roles, is_truncated, marker) = store.list_roles(path_prefix, pagination).await?;

        let response = ListRolesResponse {
            roles,
            is_truncated,
            marker,
        };

        Ok(AmiResponse::success(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::IamClient;
    use crate::store::memory::InMemoryStore;

    fn create_test_client() -> IamClient<InMemoryStore> {
        let store = InMemoryStore::new();
        IamClient::new(store)
    }

    #[tokio::test]
    async fn test_create_role() {
        let mut client = create_test_client();

        let trust_policy = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"ec2.amazonaws.com"},"Action":"sts:AssumeRole"}]}"#;

        let request = CreateRoleRequest {
            role_name: "EC2-Role".to_string(),
            assume_role_policy_document: trust_policy.to_string(),
            path: Some("/service/".to_string()),
            description: Some("EC2 service role".to_string()),
            max_session_duration: Some(3600),
            permissions_boundary: None,
            tags: None,
        };

        let response = client.create_role(request).await.unwrap();
        assert!(response.success);

        let role = response.data.unwrap();
        assert_eq!(role.role_name, "EC2-Role");
        assert!(role.role_id.starts_with("AROA"));
        assert!(role.arn.contains("/service/EC2-Role"));
        assert_eq!(role.max_session_duration, Some(3600));
    }

    #[tokio::test]
    async fn test_create_role_invalid_policy() {
        let mut client = create_test_client();

        let request = CreateRoleRequest {
            role_name: "TestRole".to_string(),
            assume_role_policy_document: "not valid json".to_string(),
            path: None,
            description: None,
            max_session_duration: None,
            permissions_boundary: None,
            tags: None,
        };

        let result = client.create_role(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_role_invalid_duration() {
        let mut client = create_test_client();

        let request = CreateRoleRequest {
            role_name: "TestRole".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
            path: None,
            description: None,
            max_session_duration: Some(1000), // Too short
            permissions_boundary: None,
            tags: None,
        };

        let result = client.create_role(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_role() {
        let mut client = create_test_client();

        let request = CreateRoleRequest {
            role_name: "MyRole".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
            path: None,
            description: Some("Test".to_string()),
            max_session_duration: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_role(request).await.unwrap();

        let response = client.get_role("MyRole".to_string()).await.unwrap();
        let role = response.data.unwrap();
        assert_eq!(role.role_name, "MyRole");
    }

    #[tokio::test]
    async fn test_get_nonexistent_role() {
        let mut client = create_test_client();

        let result = client.get_role("NonExistent".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_role() {
        let mut client = create_test_client();

        let create_request = CreateRoleRequest {
            role_name: "MyRole".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
            path: None,
            description: Some("Old description".to_string()),
            max_session_duration: Some(3600),
            permissions_boundary: None,
            tags: None,
        };
        client.create_role(create_request).await.unwrap();

        let update_request = UpdateRoleRequest {
            role_name: "MyRole".to_string(),
            description: Some("New description".to_string()),
            max_session_duration: Some(7200),
        };

        let response = client.update_role(update_request).await.unwrap();
        let role = response.data.unwrap();
        assert_eq!(role.description, Some("New description".to_string()));
        assert_eq!(role.max_session_duration, Some(7200));
    }

    #[tokio::test]
    async fn test_delete_role() {
        let mut client = create_test_client();

        let request = CreateRoleRequest {
            role_name: "ToDelete".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
            path: None,
            description: None,
            max_session_duration: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_role(request).await.unwrap();

        let result = client.delete_role("ToDelete".to_string()).await;
        assert!(result.is_ok());

        // Verify it's deleted
        let get_result = client.get_role("ToDelete".to_string()).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_list_roles() {
        let mut client = create_test_client();

        // Create multiple roles
        for i in 1..=3 {
            let request = CreateRoleRequest {
                role_name: format!("Role{}", i),
                assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
                path: Some("/service/".to_string()),
                description: None,
                max_session_duration: None,
                permissions_boundary: None,
                tags: None,
            };
            client.create_role(request).await.unwrap();
        }

        let response = client.list_roles(None).await.unwrap();
        let list_response = response.data.unwrap();

        assert_eq!(list_response.roles.len(), 3);
        assert!(!list_response.is_truncated);
    }

    #[tokio::test]
    async fn test_list_roles_with_path_prefix() {
        let mut client = create_test_client();

        // Create roles with different paths
        let request1 = CreateRoleRequest {
            role_name: "ServiceRole".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
            path: Some("/service/".to_string()),
            description: None,
            max_session_duration: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_role(request1).await.unwrap();

        let request2 = CreateRoleRequest {
            role_name: "AppRole".to_string(),
            assume_role_policy_document: r#"{"Version":"2012-10-17"}"#.to_string(),
            path: Some("/application/".to_string()),
            description: None,
            max_session_duration: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_role(request2).await.unwrap();

        let list_request = ListRolesRequest {
            path_prefix: Some("/service/".to_string()),
            pagination: None,
        };
        let response = client.list_roles(Some(list_request)).await.unwrap();
        let list_response = response.data.unwrap();

        assert_eq!(list_response.roles.len(), 1);
        assert_eq!(list_response.roles[0].role_name, "ServiceRole");
    }
}
