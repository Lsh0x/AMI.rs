//! Inline Policy Response Types

use serde::{Deserialize, Serialize};

// User inline policy responses

/// Response for put user policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutUserPolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for get user policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserPolicyResponse {
    /// The user name
    pub user_name: String,
    /// The policy name
    pub policy_name: String,
    /// The policy document in JSON format
    pub policy_document: String,
}

/// Response for delete user policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserPolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for list user policies operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUserPoliciesResponse {
    /// List of inline policy names
    pub policy_names: Vec<String>,
}

// Group inline policy responses

/// Response for put group policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutGroupPolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for get group policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGroupPolicyResponse {
    /// The group name
    pub group_name: String,
    /// The policy name
    pub policy_name: String,
    /// The policy document in JSON format
    pub policy_document: String,
}

/// Response for delete group policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteGroupPolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for list group policies operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGroupPoliciesResponse {
    /// List of inline policy names
    pub policy_names: Vec<String>,
}

// Role inline policy responses

/// Response for put role policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutRolePolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for get role policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRolePolicyResponse {
    /// The role name
    pub role_name: String,
    /// The policy name
    pub policy_name: String,
    /// The policy document in JSON format
    pub policy_document: String,
}

/// Response for delete role policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRolePolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for list role policies operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolePoliciesResponse {
    /// List of inline policy names
    pub policy_names: Vec<String>,
}
