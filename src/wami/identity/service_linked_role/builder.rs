//! Service-Linked Role Builder

use super::model::*;

/// Build a deletion task info
pub fn build_deletion_task(role_name: String) -> DeletionTaskInfo {
    let deletion_task_id = uuid::Uuid::new_v4().to_string();

    DeletionTaskInfo {
        deletion_task_id,
        status: DeletionTaskStatus::InProgress,
        role_name,
        failure_reason: None,
        create_date: chrono::Utc::now(),
    }
}
