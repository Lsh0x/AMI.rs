//! Service Linked Role Request and Response Types

use crate::wami::identity::Role;
use serde::{Deserialize, Serialize};

use super::model::*;

/// Request parameters for creating a service-linked role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceLinkedRoleRequest {
    /// The service principal for the AWS service to which this role is attached
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

/// Response for getting deletion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServiceLinkedRoleDeletionStatusResponse {
    /// Status of the deletion task
    pub status: DeletionTaskStatus,
    /// Additional information about the deletion task
    pub deletion_task_info: DeletionTaskInfo,
}
