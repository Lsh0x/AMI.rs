//! Tag Request Types

use crate::types::Tag;
use serde::{Deserialize, Serialize};

/// Request to tag an IAM resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagResourceRequest {
    /// The type of resource to tag ("user", "group", "role", "policy")
    pub resource_type: String,
    /// The identifier of the resource (user_name, group_name, role_name, or policy_arn)
    pub resource_id: String,
    /// The tags to add to the resource
    pub tags: Vec<Tag>,
}

/// Request to untag an IAM resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UntagResourceRequest {
    /// The type of resource to untag ("user", "group", "role", "policy")
    pub resource_type: String,
    /// The identifier of the resource (user_name, group_name, role_name, or policy_arn)
    pub resource_id: String,
    /// The tag keys to remove from the resource
    pub tag_keys: Vec<String>,
}

/// Request to list tags for an IAM resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourceTagsRequest {
    /// The type of resource ("user", "group", "role", "policy")
    pub resource_type: String,
    /// The identifier of the resource (user_name, group_name, role_name, or policy_arn)
    pub resource_id: String,
}
