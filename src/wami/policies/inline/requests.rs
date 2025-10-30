//! Inline Policy Request Types

use serde::{Deserialize, Serialize};

// User inline policy requests

/// Request to put an inline policy on a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutUserPolicyRequest {
    /// The name of the user
    pub user_name: String,
    /// The name of the policy
    pub policy_name: String,
    /// The policy document in JSON format
    pub policy_document: String,
}

/// Request to get an inline policy from a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserPolicyRequest {
    /// The name of the user
    pub user_name: String,
    /// The name of the policy
    pub policy_name: String,
}

/// Request to delete an inline policy from a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserPolicyRequest {
    /// The name of the user
    pub user_name: String,
    /// The name of the policy
    pub policy_name: String,
}

/// Request to list inline policies for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUserPoliciesRequest {
    /// The name of the user
    pub user_name: String,
}

// Group inline policy requests

/// Request to put an inline policy on a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutGroupPolicyRequest {
    /// The name of the group
    pub group_name: String,
    /// The name of the policy
    pub policy_name: String,
    /// The policy document in JSON format
    pub policy_document: String,
}

/// Request to get an inline policy from a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGroupPolicyRequest {
    /// The name of the group
    pub group_name: String,
    /// The name of the policy
    pub policy_name: String,
}

/// Request to delete an inline policy from a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteGroupPolicyRequest {
    /// The name of the group
    pub group_name: String,
    /// The name of the policy
    pub policy_name: String,
}

/// Request to list inline policies for a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGroupPoliciesRequest {
    /// The name of the group
    pub group_name: String,
}

// Role inline policy requests

/// Request to put an inline policy on a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutRolePolicyRequest {
    /// The name of the role
    pub role_name: String,
    /// The name of the policy
    pub policy_name: String,
    /// The policy document in JSON format
    pub policy_document: String,
}

/// Request to get an inline policy from a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRolePolicyRequest {
    /// The name of the role
    pub role_name: String,
    /// The name of the policy
    pub policy_name: String,
}

/// Request to delete an inline policy from a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRolePolicyRequest {
    /// The name of the role
    pub role_name: String,
    /// The name of the policy
    pub policy_name: String,
}

/// Request to list inline policies for a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolePoliciesRequest {
    /// The name of the role
    pub role_name: String,
}
