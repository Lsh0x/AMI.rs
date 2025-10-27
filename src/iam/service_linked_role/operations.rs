//! Service Linked Role Operations

use super::model::*;
use super::requests::*;
use crate::error::Result;
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;

impl<S: Store> IamClient<S>
where
    S::IamStore: IamStore,
{
    /// Creates a service-linked role for an AWS service
    pub async fn create_service_linked_role(
        &mut self,
        request: CreateServiceLinkedRoleRequest,
    ) -> Result<AmiResponse<CreateServiceLinkedRoleResponse>> {
        let account_id = self.account_id().await?;
        let provider = self.cloud_provider();

        let role = super::builder::build_service_linked_role(
            &request.aws_service_name,
            request.description,
            request.custom_suffix.as_deref(),
            provider.as_ref(),
            &account_id,
        );

        let store = self.iam_store().await?;
        let created_role = store.create_role(role).await?;

        Ok(AmiResponse::success(CreateServiceLinkedRoleResponse {
            role: created_role,
        }))
    }

    /// Deletes a service-linked role
    pub async fn delete_service_linked_role(
        &mut self,
        request: DeleteServiceLinkedRoleRequest,
    ) -> Result<AmiResponse<DeleteServiceLinkedRoleResponse>> {
        let store = self.iam_store().await?;

        let role = store.get_role(&request.role_name).await?.ok_or_else(|| {
            crate::error::AmiError::ResourceNotFound {
                resource: format!("Role {} not found", request.role_name),
            }
        })?;

        if !role.path.starts_with("/aws-service-role/") {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!("Role {} is not a service-linked role", request.role_name),
            });
        }

        let deletion_task_id = uuid::Uuid::new_v4().to_string();
        let deletion_task = DeletionTaskInfo {
            deletion_task_id: deletion_task_id.clone(),
            status: DeletionTaskStatus::InProgress,
            role_name: request.role_name.clone(),
            failure_reason: None,
            create_date: chrono::Utc::now(),
        };

        store
            .create_service_linked_role_deletion_task(deletion_task)
            .await?;

        Ok(AmiResponse::success(DeleteServiceLinkedRoleResponse {
            deletion_task_id,
        }))
    }

    /// Gets the status of a service-linked role deletion
    pub async fn get_service_linked_role_deletion_status(
        &mut self,
        request: GetServiceLinkedRoleDeletionStatusRequest,
    ) -> Result<AmiResponse<GetServiceLinkedRoleDeletionStatusResponse>> {
        let store = self.iam_store().await?;

        let mut deletion_task = store
            .get_service_linked_role_deletion_task(&request.deletion_task_id)
            .await?;

        if deletion_task.status == DeletionTaskStatus::InProgress {
            if store.get_role(&deletion_task.role_name).await.is_ok() {
                store.delete_role(&deletion_task.role_name).await?;
                deletion_task.status = DeletionTaskStatus::Succeeded;
                store
                    .update_service_linked_role_deletion_task(deletion_task.clone())
                    .await?;
            } else {
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

    #[tokio::test]
    async fn test_create_service_linked_role() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

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
    }
}
