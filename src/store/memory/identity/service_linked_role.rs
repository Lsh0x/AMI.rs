//! Service-Linked Role Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::ServiceLinkedRoleStore;
use crate::wami::identity::service_linked_role::DeletionTaskInfo;
use async_trait::async_trait;

#[async_trait]
impl ServiceLinkedRoleStore for InMemoryWamiStore {
    async fn create_service_linked_role_deletion_task(
        &mut self,
        task: DeletionTaskInfo,
    ) -> Result<()> {
        self.service_linked_role_deletion_tasks
            .insert(task.deletion_task_id.clone(), task);
        Ok(())
    }

    async fn get_service_linked_role_deletion_task(
        &self,
        deletion_task_id: &str,
    ) -> Result<Option<DeletionTaskInfo>> {
        Ok(self
            .service_linked_role_deletion_tasks
            .get(deletion_task_id)
            .cloned())
    }
}
