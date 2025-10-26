//! Policy Evaluation Domain Models

use serde::{Deserialize, Serialize};

/// Result of a policy simulation for a single action/resource combination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationResult {
    /// The action that was evaluated
    pub eval_action_name: String,
    /// The resource that was evaluated
    pub eval_resource_name: String,
    /// The evaluation decision ("allowed" or "denied")
    pub eval_decision: String,
    /// List of statements that matched
    pub matched_statements: Vec<StatementMatch>,
    /// List of statements that didn't match
    pub missing_context_values: Vec<String>,
}

/// Information about a policy statement that matched
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatementMatch {
    /// The policy document that contained this statement (if available)
    pub source_policy_id: Option<String>,
    /// The effect of the statement ("Allow" or "Deny")
    pub effect: String,
    /// Whether this statement matched the action
    pub matched_action: bool,
    /// Whether this statement matched the resource
    pub matched_resource: bool,
}

/// Context entry for policy condition evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    /// The key for the context entry (e.g., "aws:CurrentTime")
    pub context_key_name: String,
    /// The value for the context entry
    pub context_key_values: Vec<String>,
    /// The data type (String, StringList, Numeric, Boolean, etc.)
    pub context_key_type: String,
}
