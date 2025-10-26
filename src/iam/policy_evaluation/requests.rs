//! Policy Evaluation Request and Response Types

use serde::{Deserialize, Serialize};

use super::model::*;

/// Request to simulate a custom policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateCustomPolicyRequest {
    /// List of policy documents to simulate (JSON strings)
    pub policy_input_list: Vec<String>,
    /// List of actions to simulate (e.g., ["s3:GetObject", "s3:PutObject"])
    pub action_names: Vec<String>,
    /// List of resources to simulate (ARNs or patterns)
    pub resource_arns: Option<Vec<String>>,
    /// Optional context entries for condition evaluation
    pub context_entries: Option<Vec<ContextEntry>>,
}

/// Request to simulate a principal's policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatePrincipalPolicyRequest {
    /// The ARN of the principal (user, group, or role) whose policies to simulate
    pub policy_source_arn: String,
    /// List of actions to simulate
    pub action_names: Vec<String>,
    /// List of resources to simulate (ARNs or patterns)
    pub resource_arns: Option<Vec<String>>,
    /// Optional additional policy documents to include in simulation
    pub policy_input_list: Option<Vec<String>>,
    /// Optional context entries for condition evaluation
    pub context_entries: Option<Vec<ContextEntry>>,
}

/// Response from policy simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatePolicyResponse {
    /// The evaluation results
    pub evaluation_results: Vec<EvaluationResult>,
    /// Whether there are more results (for pagination)
    pub is_truncated: bool,
}
