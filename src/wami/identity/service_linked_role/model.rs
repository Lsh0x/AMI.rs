//! Service Linked Role Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub create_date: DateTime<Utc>,
}
