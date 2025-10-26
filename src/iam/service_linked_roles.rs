use crate::error::Result;
use crate::iam::Role;
use crate::provider::ResourceType;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;
use serde::{Deserialize, Serialize};

/// Request parameters for creating a service-linked role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceLinkedRoleRequest {
    /// The service principal for the AWS service to which this role is attached
    /// Example: "elasticbeanstalk.amazonaws.com"
    pub aws_service_name: String,
    /// A description of the role
    pub description: Option<String>,
    /// A custom suffix for the service-linked role name
    pub custom_suffix: Option<String>,
}

/// Response for creating a service-linked role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceLinkedRoleResponse {
    /// The role that was created
    pub role: Role,
}

/// Request parameters for deleting a service-linked role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteServiceLinkedRoleRequest {
    /// The name of the service-linked role to delete
    pub role_name: String,
}

/// Response for deleting a service-linked role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteServiceLinkedRoleResponse {
    /// The deletion task identifier that can be used to check the status
    pub deletion_task_id: String,
}

/// Request parameters for getting the deletion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServiceLinkedRoleDeletionStatusRequest {
    /// The deletion task identifier
    pub deletion_task_id: String,
}

/// Status of a service-linked role deletion task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeletionTaskStatus {
    /// The deletion has been initiated
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    /// The deletion has succeeded
    #[serde(rename = "SUCCEEDED")]
    Succeeded,
    /// The deletion has failed
    #[serde(rename = "FAILED")]
    Failed,
    /// The deletion is not started
    #[serde(rename = "NOT_STARTED")]
    NotStarted,
}

/// Reason for deletion failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionTaskFailureReason {
    /// Reason description
    pub reason: String,
    /// List of role usage types that prevented deletion
    pub role_usage_list: Vec<RoleUsageType>,
}

/// Information about where the role is being used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleUsageType {
    /// The AWS region where the role is being used
    pub region: Option<String>,
    /// Resources using the role
    pub resources: Vec<String>,
}

/// Deletion task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionTaskInfo {
    /// The deletion task identifier
    pub deletion_task_id: String,
    /// The status of the deletion task
    pub status: DeletionTaskStatus,
    /// The role name being deleted
    pub role_name: String,
    /// Failure reason if status is Failed
    pub failure_reason: Option<DeletionTaskFailureReason>,
    /// The date and time when the deletion task was created
    pub create_date: chrono::DateTime<chrono::Utc>,
}

/// Response for getting deletion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServiceLinkedRoleDeletionStatusResponse {
    /// Status of the deletion task
    pub status: DeletionTaskStatus,
    /// Additional information about the deletion task
    pub deletion_task_info: DeletionTaskInfo,
}

impl<S: Store> crate::iam::IamClient<S>
where
    S::IamStore: IamStore,
{
    /// Creates a service-linked role for an AWS service
    ///
    /// Service-linked roles are predefined roles that grant permissions to AWS services
    /// so they can perform actions on your behalf. These roles are directly linked to
    /// a specific AWS service and include all the permissions that the service requires.
    ///
    /// # Arguments
    ///
    /// * `request` - The service-linked role creation request
    ///
    /// # Returns
    ///
    /// Returns the created service-linked role
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateServiceLinkedRoleRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = CreateServiceLinkedRoleRequest {
    ///     aws_service_name: "elasticbeanstalk.amazonaws.com".to_string(),
    ///     description: Some("Service role for Elastic Beanstalk".to_string()),
    ///     custom_suffix: None,
    /// };
    ///
    /// let response = client.create_service_linked_role(request).await?;
    /// let role = response.data.unwrap().role;
    /// println!("Created service-linked role: {}", role.role_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_service_linked_role(
        &mut self,
        request: CreateServiceLinkedRoleRequest,
    ) -> Result<AmiResponse<CreateServiceLinkedRoleResponse>> {
        let store = self.iam_store().await?;
        let account_id = store.account_id();

        // Extract service name from the service principal
        // e.g., "elasticbeanstalk.amazonaws.com" -> "elasticbeanstalk"
        let service_name = request.aws_service_name.split('.').next().ok_or_else(|| {
            crate::error::AmiError::InvalidParameter {
                message: "Invalid AWS service name".to_string(),
            }
        })?;

        // Convert to PascalCase for role name (e.g., "elasticbeanstalk" -> "ElasticBeanstalk")
        let service_name_pascal = service_name
            .split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<String>();

        // Generate role name: AWSServiceRoleForServiceName[CustomSuffix]
        let role_name = if let Some(suffix) = &request.custom_suffix {
            format!("AWSServiceRoleFor{}_{}", service_name_pascal, suffix)
        } else {
            format!("AWSServiceRoleFor{}", service_name_pascal)
        };

        let provider = store.cloud_provider();

        // Override role_name with provider-specific naming if custom suffix is not provided
        let role_name = if request.custom_suffix.is_none() {
            provider.generate_service_linked_role_name(&request.aws_service_name, None)
        } else {
            role_name
        };

        // Service-linked roles have a specific path pattern (provider-specific)
        let path = provider.generate_service_linked_role_path(&request.aws_service_name);

        // Generate the assume role policy document that allows the service to assume this role
        let assume_role_policy_document = serde_json::json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {
                    "Service": request.aws_service_name
                },
                "Action": "sts:AssumeRole"
            }]
        })
        .to_string();

        // Use provider for role ID generation
        let role_id = provider.generate_resource_id(ResourceType::ServiceLinkedRole);

        // Use provider for ARN generation
        let arn = provider.generate_resource_identifier(
            ResourceType::ServiceLinkedRole,
            account_id,
            &path,
            &role_name,
        );

        let role = Role {
            role_name: role_name.clone(),
            role_id,
            arn,
            path,
            create_date: chrono::Utc::now(),
            assume_role_policy_document,
            description: request.description,
            max_session_duration: Some(3600), // Default to 1 hour
            permissions_boundary: None,
            tags: vec![],
        };

        let created_role = store.create_role(role).await?;

        Ok(AmiResponse::success(CreateServiceLinkedRoleResponse {
            role: created_role,
        }))
    }

    /// Deletes a service-linked role
    ///
    /// Submits a deletion request for a service-linked role. The deletion may not happen
    /// immediately if the role is still being used by AWS resources. The deletion is
    /// asynchronous, and you can check its status using the returned deletion task ID.
    ///
    /// # Arguments
    ///
    /// * `request` - The deletion request
    ///
    /// # Returns
    ///
    /// Returns a deletion task ID that can be used to check the deletion status
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, DeleteServiceLinkedRoleRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = wami::create_memory_store();
    /// # let mut client = MemoryIamClient::new(store);
    /// let request = DeleteServiceLinkedRoleRequest {
    ///     role_name: "AWSServiceRoleForElasticBeanstalk".to_string(),
    /// };
    ///
    /// let response = client.delete_service_linked_role(request).await?;
    /// let deletion_task_id = response.data.unwrap().deletion_task_id;
    /// println!("Deletion task ID: {}", deletion_task_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_service_linked_role(
        &mut self,
        request: DeleteServiceLinkedRoleRequest,
    ) -> Result<AmiResponse<DeleteServiceLinkedRoleResponse>> {
        let store = self.iam_store().await?;

        // Check if the role exists
        let role = store.get_role(&request.role_name).await?.ok_or_else(|| {
            crate::error::AmiError::ResourceNotFound {
                resource: format!("Role {} not found", request.role_name),
            }
        })?;

        // Verify it's a service-linked role (path starts with /aws-service-role/)
        if !role.path.starts_with("/aws-service-role/") {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!("Role {} is not a service-linked role", request.role_name),
            });
        }

        // Generate deletion task ID
        let deletion_task_id = uuid::Uuid::new_v4().to_string();

        // Create deletion task info
        let deletion_task = DeletionTaskInfo {
            deletion_task_id: deletion_task_id.clone(),
            status: DeletionTaskStatus::InProgress,
            role_name: request.role_name.clone(),
            failure_reason: None,
            create_date: chrono::Utc::now(),
        };

        // Store the deletion task
        store
            .create_service_linked_role_deletion_task(deletion_task)
            .await?;

        // In a real implementation, we would check if the role is in use
        // For now, we'll just mark it for deletion and complete it immediately
        // The role will be deleted asynchronously

        Ok(AmiResponse::success(DeleteServiceLinkedRoleResponse {
            deletion_task_id,
        }))
    }

    /// Gets the status of a service-linked role deletion
    ///
    /// Retrieves the status of the deletion of a service-linked role.
    ///
    /// # Arguments
    ///
    /// * `request` - The request containing the deletion task ID
    ///
    /// # Returns
    ///
    /// Returns the current status of the deletion task
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, GetServiceLinkedRoleDeletionStatusRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = wami::create_memory_store();
    /// # let mut client = MemoryIamClient::new(store);
    /// # let deletion_task_id = "task-123".to_string();
    /// let request = GetServiceLinkedRoleDeletionStatusRequest {
    ///     deletion_task_id: deletion_task_id.clone(),
    /// };
    ///
    /// let response = client.get_service_linked_role_deletion_status(request).await?;
    /// let status = response.data.unwrap().status;
    /// println!("Deletion status: {:?}", status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_service_linked_role_deletion_status(
        &mut self,
        request: GetServiceLinkedRoleDeletionStatusRequest,
    ) -> Result<AmiResponse<GetServiceLinkedRoleDeletionStatusResponse>> {
        let store = self.iam_store().await?;

        let mut deletion_task = store
            .get_service_linked_role_deletion_task(&request.deletion_task_id)
            .await?;

        // Simulate async deletion completion
        // In a real implementation, this would check actual AWS resources
        if deletion_task.status == DeletionTaskStatus::InProgress {
            // Check if the role still exists
            if store.get_role(&deletion_task.role_name).await.is_ok() {
                // Complete the deletion
                store.delete_role(&deletion_task.role_name).await?;
                deletion_task.status = DeletionTaskStatus::Succeeded;
                store
                    .update_service_linked_role_deletion_task(deletion_task.clone())
                    .await?;
            } else {
                // Role was already deleted
                deletion_task.status = DeletionTaskStatus::Succeeded;
                store
                    .update_service_linked_role_deletion_task(deletion_task.clone())
                    .await?;
            }
        }

        Ok(AmiResponse::success(
            GetServiceLinkedRoleDeletionStatusResponse {
                status: deletion_task.status.clone(),
                deletion_task_info: deletion_task,
            },
        ))
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
    async fn test_create_service_linked_role() {
        let mut client = create_test_client();

        let request = CreateServiceLinkedRoleRequest {
            aws_service_name: "elasticbeanstalk.amazonaws.com".to_string(),
            description: Some("Service role for Elastic Beanstalk".to_string()),
            custom_suffix: None,
        };

        let response = client.create_service_linked_role(request).await.unwrap();
        assert!(response.success);

        let role = response.data.unwrap().role;
        assert_eq!(role.role_name, "AWSServiceRoleForElasticbeanstalk");
        assert!(role.path.starts_with("/aws-service-role/"));
        assert!(role.path.contains("elasticbeanstalk.amazonaws.com"));
        assert!(role.role_id.starts_with("AROA"));
    }

    #[tokio::test]
    async fn test_create_service_linked_role_with_custom_suffix() {
        let mut client = create_test_client();

        let request = CreateServiceLinkedRoleRequest {
            aws_service_name: "lex.amazonaws.com".to_string(),
            description: Some("Lex bot role".to_string()),
            custom_suffix: Some("MyBot".to_string()),
        };

        let response = client.create_service_linked_role(request).await.unwrap();
        assert!(response.success);

        let role = response.data.unwrap().role;
        assert_eq!(role.role_name, "AWSServiceRoleForLex_MyBot");
    }

    #[tokio::test]
    async fn test_create_service_linked_role_with_hyphenated_service() {
        let mut client = create_test_client();

        let request = CreateServiceLinkedRoleRequest {
            aws_service_name: "elasticache.amazonaws.com".to_string(),
            description: None,
            custom_suffix: None,
        };

        let response = client.create_service_linked_role(request).await.unwrap();
        assert!(response.success);

        let role = response.data.unwrap().role;
        assert_eq!(role.role_name, "AWSServiceRoleForElasticache");
        assert!(role.arn.contains("role/aws-service-role"));
    }

    #[tokio::test]
    async fn test_delete_service_linked_role() {
        let mut client = create_test_client();

        // First create a service-linked role
        let create_request = CreateServiceLinkedRoleRequest {
            aws_service_name: "elasticbeanstalk.amazonaws.com".to_string(),
            description: None,
            custom_suffix: None,
        };

        let create_response = client
            .create_service_linked_role(create_request)
            .await
            .unwrap();
        let role_name = create_response.data.unwrap().role.role_name;

        // Delete the role
        let delete_request = DeleteServiceLinkedRoleRequest {
            role_name: role_name.clone(),
        };

        let delete_response = client
            .delete_service_linked_role(delete_request)
            .await
            .unwrap();
        assert!(delete_response.success);

        let deletion_task_id = delete_response.data.unwrap().deletion_task_id;
        assert!(!deletion_task_id.is_empty());
    }

    #[tokio::test]
    async fn test_delete_non_service_linked_role() {
        let mut client = create_test_client();

        // Create a regular role (not service-linked)
        let trust_policy = r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"ec2.amazonaws.com"},"Action":"sts:AssumeRole"}]}"#;

        let create_request = crate::iam::roles::CreateRoleRequest {
            role_name: "RegularRole".to_string(),
            assume_role_policy_document: trust_policy.to_string(),
            path: Some("/".to_string()),
            description: None,
            max_session_duration: None,
            permissions_boundary: None,
            tags: None,
        };

        client.create_role(create_request).await.unwrap();

        // Try to delete it as a service-linked role (should fail)
        let delete_request = DeleteServiceLinkedRoleRequest {
            role_name: "RegularRole".to_string(),
        };

        let result = client.delete_service_linked_role(delete_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_deletion_status() {
        let mut client = create_test_client();

        // Create and delete a service-linked role
        let create_request = CreateServiceLinkedRoleRequest {
            aws_service_name: "elasticbeanstalk.amazonaws.com".to_string(),
            description: None,
            custom_suffix: None,
        };

        let create_response = client
            .create_service_linked_role(create_request)
            .await
            .unwrap();
        let role_name = create_response.data.unwrap().role.role_name;

        let delete_request = DeleteServiceLinkedRoleRequest {
            role_name: role_name.clone(),
        };

        let delete_response = client
            .delete_service_linked_role(delete_request)
            .await
            .unwrap();
        let deletion_task_id = delete_response.data.unwrap().deletion_task_id;

        // Check the deletion status
        let status_request = GetServiceLinkedRoleDeletionStatusRequest {
            deletion_task_id: deletion_task_id.clone(),
        };

        let status_response = client
            .get_service_linked_role_deletion_status(status_request)
            .await
            .unwrap();
        assert!(status_response.success);

        let status = status_response.data.unwrap().status;
        // Status should be either InProgress or Succeeded
        assert!(
            status == DeletionTaskStatus::InProgress || status == DeletionTaskStatus::Succeeded
        );
    }

    #[tokio::test]
    async fn test_deletion_completes_on_status_check() {
        let mut client = create_test_client();

        // Create and delete a service-linked role
        let create_request = CreateServiceLinkedRoleRequest {
            aws_service_name: "elasticbeanstalk.amazonaws.com".to_string(),
            description: None,
            custom_suffix: None,
        };

        let create_response = client
            .create_service_linked_role(create_request)
            .await
            .unwrap();
        let role_name = create_response.data.unwrap().role.role_name;

        let delete_request = DeleteServiceLinkedRoleRequest {
            role_name: role_name.clone(),
        };

        let delete_response = client
            .delete_service_linked_role(delete_request)
            .await
            .unwrap();
        let deletion_task_id = delete_response.data.unwrap().deletion_task_id;

        // First status check should trigger completion
        let status_request = GetServiceLinkedRoleDeletionStatusRequest {
            deletion_task_id: deletion_task_id.clone(),
        };

        let status_response = client
            .get_service_linked_role_deletion_status(status_request.clone())
            .await
            .unwrap();

        let status = status_response.data.unwrap().status;
        assert_eq!(status, DeletionTaskStatus::Succeeded);

        // Subsequent checks should still return Succeeded
        let status_response2 = client
            .get_service_linked_role_deletion_status(status_request)
            .await
            .unwrap();

        let status2 = status_response2.data.unwrap().status;
        assert_eq!(status2, DeletionTaskStatus::Succeeded);
    }
}
