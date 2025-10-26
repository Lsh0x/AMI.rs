use crate::error::Result;
use crate::iam::IamClient;
use crate::store::Store;
use crate::types::{AmiResponse, PolicyDocument};
use serde::{Deserialize, Serialize};

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

/// Response from policy simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatePolicyResponse {
    /// The evaluation results
    pub evaluation_results: Vec<EvaluationResult>,
    /// Whether there are more results (for pagination)
    pub is_truncated: bool,
}

impl<S: Store> IamClient<S> {
    /// Simulate a custom IAM policy
    ///
    /// This method simulates the effect of IAM policies without making actual requests.
    /// It's useful for testing policy logic before deployment.
    ///
    /// # Arguments
    ///
    /// * `request` - The simulation request with policies and actions to test
    ///
    /// # Returns
    ///
    /// Returns evaluation results for each action/resource combination
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, SimulateCustomPolicyRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let policy = r#"{
    ///     "Version": "2012-10-17",
    ///     "Statement": [{
    ///         "Effect": "Allow",
    ///         "Action": "s3:GetObject",
    ///         "Resource": "arn:aws:s3:::my-bucket/*"
    ///     }]
    /// }"#;
    ///
    /// let request = SimulateCustomPolicyRequest {
    ///     policy_input_list: vec![policy.to_string()],
    ///     action_names: vec!["s3:GetObject".to_string()],
    ///     resource_arns: Some(vec!["arn:aws:s3:::my-bucket/file.txt".to_string()]),
    ///     context_entries: None,
    /// };
    ///
    /// let response = client.simulate_custom_policy(request).await?;
    /// let result = &response.data.unwrap().evaluation_results[0];
    /// assert_eq!(result.eval_decision, "allowed");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn simulate_custom_policy(
        &mut self,
        request: SimulateCustomPolicyRequest,
    ) -> Result<AmiResponse<SimulatePolicyResponse>> {
        // Parse policy documents
        let mut policy_documents = Vec::new();
        for policy_json in &request.policy_input_list {
            match serde_json::from_str::<PolicyDocument>(policy_json) {
                Ok(doc) => policy_documents.push(doc),
                Err(_) => {
                    return Err(crate::error::AmiError::InvalidParameter {
                        message: "Invalid policy document JSON".to_string(),
                    });
                }
            }
        }

        // Default resource if not specified
        let resources = request
            .resource_arns
            .unwrap_or_else(|| vec!["*".to_string()]);

        // Evaluate each action/resource combination
        let mut evaluation_results = Vec::new();
        for action in &request.action_names {
            for resource in &resources {
                let result = Self::evaluate_policy(
                    &policy_documents,
                    action,
                    resource,
                    request.context_entries.as_ref(),
                );
                evaluation_results.push(result);
            }
        }

        let response = SimulatePolicyResponse {
            evaluation_results,
            is_truncated: false,
        };

        Ok(AmiResponse::success(response))
    }

    /// Simulate a principal's IAM policy
    ///
    /// Similar to `simulate_custom_policy`, but simulates the policies attached to a principal
    /// (user, group, or role).
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, SimulatePrincipalPolicyRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create a user
    /// let user_request = CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// let user_response = client.create_user(user_request).await?;
    /// let user_arn = user_response.data.unwrap().arn;
    ///
    /// // Simulate policies (even if no policies attached, will deny by default)
    /// let request = SimulatePrincipalPolicyRequest {
    ///     policy_source_arn: user_arn,
    ///     action_names: vec!["s3:GetObject".to_string()],
    ///     resource_arns: Some(vec!["arn:aws:s3:::my-bucket/*".to_string()]),
    ///     policy_input_list: None,
    ///     context_entries: None,
    /// };
    ///
    /// let response = client.simulate_principal_policy(request).await?;
    /// let result = &response.data.unwrap().evaluation_results[0];
    /// assert_eq!(result.eval_decision, "denied"); // No policies = deny
    /// # Ok(())
    /// # }
    /// ```
    pub async fn simulate_principal_policy(
        &mut self,
        request: SimulatePrincipalPolicyRequest,
    ) -> Result<AmiResponse<SimulatePolicyResponse>> {
        // For now, we'll use the additional policy documents if provided
        // In a full implementation, we would fetch the principal's attached policies
        let policy_input_list = request.policy_input_list.unwrap_or_default();

        let custom_request = SimulateCustomPolicyRequest {
            policy_input_list,
            action_names: request.action_names,
            resource_arns: request.resource_arns,
            context_entries: request.context_entries,
        };

        self.simulate_custom_policy(custom_request).await
    }

    /// Evaluate a policy for a specific action and resource
    ///
    /// AWS IAM evaluation logic:
    /// 1. By default, all requests are denied (implicit deny)
    /// 2. An explicit allow overrides the default deny
    /// 3. An explicit deny overrides any allows
    fn evaluate_policy(
        policy_documents: &[PolicyDocument],
        action: &str,
        resource: &str,
        _context: Option<&Vec<ContextEntry>>,
    ) -> EvaluationResult {
        let mut has_allow = false;
        let mut has_deny = false;
        let mut matched_statements = Vec::new();

        for policy_doc in policy_documents {
            for statement in &policy_doc.statement {
                let action_matches = Self::matches_pattern(&statement.action, action);
                let resource_matches = Self::matches_pattern(&statement.resource, resource);

                if action_matches && resource_matches {
                    let statement_match = StatementMatch {
                        source_policy_id: None,
                        effect: statement.effect.clone(),
                        matched_action: action_matches,
                        matched_resource: resource_matches,
                    };
                    matched_statements.push(statement_match);

                    match statement.effect.as_str() {
                        "Allow" => has_allow = true,
                        "Deny" => has_deny = true,
                        _ => {}
                    }
                }
            }
        }

        // Deny always wins
        let decision = if has_deny {
            "denied".to_string()
        } else if has_allow {
            "allowed".to_string()
        } else {
            // Implicit deny
            "denied".to_string()
        };

        EvaluationResult {
            eval_action_name: action.to_string(),
            eval_resource_name: resource.to_string(),
            eval_decision: decision,
            matched_statements,
            missing_context_values: Vec::new(),
        }
    }

    /// Check if a value matches a pattern (supports wildcards)
    ///
    /// AWS IAM supports wildcards:
    /// - `*` matches zero or more characters
    /// - `?` matches exactly one character
    fn matches_pattern(patterns: &[String], value: &str) -> bool {
        for pattern in patterns {
            if Self::matches_single_pattern(pattern, value) {
                return true;
            }
        }
        false
    }

    /// Check if a value matches a single pattern
    fn matches_single_pattern(pattern: &str, value: &str) -> bool {
        // Simple wildcard matching
        if pattern == "*" {
            return true;
        }

        // Convert pattern to regex-like matching
        // For simplicity, we only support * wildcard
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            let mut current_pos = 0;

            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }

                // First part must match at the beginning
                if i == 0 {
                    if !value[current_pos..].starts_with(part) {
                        return false;
                    }
                    current_pos += part.len();
                } else if i == parts.len() - 1 {
                    // Last part must match at the end
                    if !value[current_pos..].ends_with(part) {
                        return false;
                    }
                } else {
                    // Middle parts must exist somewhere
                    if let Some(pos) = value[current_pos..].find(part) {
                        current_pos += pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            true
        } else {
            // Exact match
            pattern == value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::user::CreateUserRequest;
    use crate::store::memory::InMemoryStore;

    #[tokio::test]
    async fn test_simulate_custom_policy_allow() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let policy = r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:GetObject"],
                "Resource": ["arn:aws:s3:::my-bucket/*"]
            }]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy.to_string()],
            action_names: vec!["s3:GetObject".to_string()],
            resource_arns: Some(vec!["arn:aws:s3:::my-bucket/file.txt".to_string()]),
            context_entries: None,
        };

        let response = client.simulate_custom_policy(request).await.unwrap();
        let result = &response.data.unwrap().evaluation_results[0];

        assert_eq!(result.eval_decision, "allowed");
        assert_eq!(result.matched_statements.len(), 1);
    }

    #[tokio::test]
    async fn test_simulate_custom_policy_deny() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let policy = r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:GetObject"],
                "Resource": ["arn:aws:s3:::my-bucket/*"]
            }]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy.to_string()],
            action_names: vec!["s3:PutObject".to_string()], // Different action
            resource_arns: Some(vec!["arn:aws:s3:::my-bucket/file.txt".to_string()]),
            context_entries: None,
        };

        let response = client.simulate_custom_policy(request).await.unwrap();
        let result = &response.data.unwrap().evaluation_results[0];

        assert_eq!(result.eval_decision, "denied");
        assert_eq!(result.matched_statements.len(), 0);
    }

    #[tokio::test]
    async fn test_simulate_explicit_deny() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let policy = r#"{
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": ["s3:*"],
                    "Resource": ["*"]
                },
                {
                    "Effect": "Deny",
                    "Action": ["s3:DeleteObject"],
                    "Resource": ["*"]
                }
            ]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy.to_string()],
            action_names: vec!["s3:DeleteObject".to_string()],
            resource_arns: Some(vec!["arn:aws:s3:::my-bucket/file.txt".to_string()]),
            context_entries: None,
        };

        let response = client.simulate_custom_policy(request).await.unwrap();
        let result = &response.data.unwrap().evaluation_results[0];

        // Explicit deny should override allow
        assert_eq!(result.eval_decision, "denied");
        assert_eq!(result.matched_statements.len(), 2);
    }

    #[tokio::test]
    async fn test_wildcard_action_matching() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let policy = r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:Get*"],
                "Resource": ["*"]
            }]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy.to_string()],
            action_names: vec![
                "s3:GetObject".to_string(),
                "s3:GetBucketLocation".to_string(),
            ],
            resource_arns: Some(vec!["arn:aws:s3:::my-bucket/file.txt".to_string()]),
            context_entries: None,
        };

        let response = client.simulate_custom_policy(request).await.unwrap();
        let results = &response.data.unwrap().evaluation_results;

        // Both should be allowed due to wildcard
        assert_eq!(results[0].eval_decision, "allowed");
        assert_eq!(results[1].eval_decision, "allowed");
    }

    #[tokio::test]
    async fn test_wildcard_resource_matching() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let policy = r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:GetObject"],
                "Resource": ["arn:aws:s3:::my-bucket/public/*"]
            }]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy.to_string()],
            action_names: vec!["s3:GetObject".to_string()],
            resource_arns: Some(vec![
                "arn:aws:s3:::my-bucket/public/file1.txt".to_string(),
                "arn:aws:s3:::my-bucket/private/file2.txt".to_string(),
            ]),
            context_entries: None,
        };

        let response = client.simulate_custom_policy(request).await.unwrap();
        let results = &response.data.unwrap().evaluation_results;

        // First should be allowed, second denied
        assert_eq!(results[0].eval_decision, "allowed");
        assert_eq!(results[1].eval_decision, "denied");
    }

    #[tokio::test]
    async fn test_simulate_principal_policy() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create a user
        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        let user_response = client.create_user(user_request).await.unwrap();
        let user_arn = user_response.data.unwrap().arn;

        // Simulate with no policies (should deny)
        let request = SimulatePrincipalPolicyRequest {
            policy_source_arn: user_arn,
            action_names: vec!["s3:GetObject".to_string()],
            resource_arns: Some(vec!["arn:aws:s3:::my-bucket/*".to_string()]),
            policy_input_list: None,
            context_entries: None,
        };

        let response = client.simulate_principal_policy(request).await.unwrap();
        let result = &response.data.unwrap().evaluation_results[0];

        // No policies = implicit deny
        assert_eq!(result.eval_decision, "denied");
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        // Test exact match
        assert!(IamClient::<InMemoryStore>::matches_single_pattern(
            "s3:GetObject",
            "s3:GetObject"
        ));

        // Test wildcard match
        assert!(IamClient::<InMemoryStore>::matches_single_pattern(
            "s3:Get*",
            "s3:GetObject"
        ));
        assert!(IamClient::<InMemoryStore>::matches_single_pattern(
            "s3:*",
            "s3:GetObject"
        ));
        assert!(IamClient::<InMemoryStore>::matches_single_pattern(
            "*", "anything"
        ));

        // Test no match
        assert!(!IamClient::<InMemoryStore>::matches_single_pattern(
            "s3:Put*",
            "s3:GetObject"
        ));
    }

    #[tokio::test]
    async fn test_multiple_policies() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let policy1 = r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:GetObject"],
                "Resource": ["*"]
            }]
        }"#;

        let policy2 = r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:PutObject"],
                "Resource": ["*"]
            }]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy1.to_string(), policy2.to_string()],
            action_names: vec!["s3:GetObject".to_string(), "s3:PutObject".to_string()],
            resource_arns: Some(vec!["arn:aws:s3:::my-bucket/file.txt".to_string()]),
            context_entries: None,
        };

        let response = client.simulate_custom_policy(request).await.unwrap();
        let results = &response.data.unwrap().evaluation_results;

        // Both actions should be allowed
        assert_eq!(results[0].eval_decision, "allowed");
        assert_eq!(results[1].eval_decision, "allowed");
    }
}
