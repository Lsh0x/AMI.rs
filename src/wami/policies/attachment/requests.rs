//! Policy Attachment Request Types

use serde::{Deserialize, Serialize};

/// Request to attach a managed policy to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachUserPolicyRequest {
    /// The name of the user
    pub user_name: String,
    /// The ARN of the policy to attach
    pub policy_arn: String,
}

/// Request to detach a managed policy from a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachUserPolicyRequest {
    /// The name of the user
    pub user_name: String,
    /// The ARN of the policy to detach
    pub policy_arn: String,
}

/// Request to list attached policies for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAttachedUserPoliciesRequest {
    /// The name of the user
    pub user_name: String,
}

/// Request to attach a managed policy to a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachGroupPolicyRequest {
    /// The name of the group
    pub group_name: String,
    /// The ARN of the policy to attach
    pub policy_arn: String,
}

/// Request to detach a managed policy from a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachGroupPolicyRequest {
    /// The name of the group
    pub group_name: String,
    /// The ARN of the policy to detach
    pub policy_arn: String,
}

/// Request to list attached policies for a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAttachedGroupPoliciesRequest {
    /// The name of the group
    pub group_name: String,
}

/// Request to attach a managed policy to a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachRolePolicyRequest {
    /// The name of the role
    pub role_name: String,
    /// The ARN of the policy to attach
    pub policy_arn: String,
}

/// Request to detach a managed policy from a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachRolePolicyRequest {
    /// The name of the role
    pub role_name: String,
    /// The ARN of the policy to detach
    pub policy_arn: String,
}

/// Request to list attached policies for a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAttachedRolePoliciesRequest {
    /// The name of the role
    pub role_name: String,
}
