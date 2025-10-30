//! Policy Attachment Response Types

use super::model::AttachedPolicy;
use serde::{Deserialize, Serialize};

/// Response for attach user policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachUserPolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for detach user policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachUserPolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for list attached user policies operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAttachedUserPoliciesResponse {
    /// List of attached policies
    pub attached_policies: Vec<AttachedPolicy>,
}

/// Response for attach group policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachGroupPolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for detach group policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachGroupPolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for list attached group policies operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAttachedGroupPoliciesResponse {
    /// List of attached policies
    pub attached_policies: Vec<AttachedPolicy>,
}

/// Response for attach role policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachRolePolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for detach role policy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachRolePolicyResponse {
    /// Success message
    pub message: String,
}

/// Response for list attached role policies operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAttachedRolePoliciesResponse {
    /// List of attached policies
    pub attached_policies: Vec<AttachedPolicy>,
}
