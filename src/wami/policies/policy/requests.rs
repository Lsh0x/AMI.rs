//! Policy Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::Policy;
use crate::types::{PaginationParams, Tag};

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
    pub tags: Option<Vec<Tag>>,
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
