//! Service-Linked Role Store Trait

use crate::error::Result;
use crate::wami::identity::service_linked_role::DeletionTaskInfo;
use async_trait::async_trait;

/// Trait for service-linked role storage operations
#[async_trait]
pub trait ServiceLinkedRoleStore: Send + Sync {
    async fn create_service_linked_role_deletion_task(
        &mut self,
        task: DeletionTaskInfo,
    ) -> Result<()>;

    async fn get_service_linked_role_deletion_task(
        &self,
        deletion_task_id: &str,
    ) -> Result<Option<DeletionTaskInfo>>;
}
