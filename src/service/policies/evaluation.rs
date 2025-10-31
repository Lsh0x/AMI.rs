//! Policy Evaluation Service
//!
//! Orchestrates policy simulation and evaluation operations.

use crate::error::{AmiError, Result};
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::{PolicyStore, RoleStore, UserStore};
use crate::types::PolicyDocument;
use crate::wami::policies::evaluation::{
    EvaluationResult, SimulateCustomPolicyRequest, SimulatePolicyResponse,
    SimulatePrincipalPolicyRequest, StatementMatch,
};
use std::sync::{Arc, RwLock};

/// Service for policy simulation and evaluation
///
/// Provides high-level operations for testing and validating IAM policies.
pub struct EvaluationService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: UserStore + RoleStore + PolicyStore> EvaluationService<S> {
    /// Create a new EvaluationService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>, account_id: String) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
            account_id,
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
            account_id: self.account_id.clone(),
        }
    }

    /// Simulate custom policy documents without creating them
    ///
    /// This is a stateless operation that evaluates policy documents directly.
    pub async fn simulate_custom_policy(
        &self,
        request: SimulateCustomPolicyRequest,
    ) -> Result<SimulatePolicyResponse> {
        // Parse policy documents
        let policies: Result<Vec<PolicyDocument>> = request
            .policy_input_list
            .iter()
            .map(|policy_str| {
                serde_json::from_str(policy_str).map_err(|e| AmiError::InvalidParameter {
                    message: format!("Invalid policy document: {}", e),
                })
            })
            .collect();

        let policies = policies?;

        // Default resource if not provided
        let resources = request
            .resource_arns
            .unwrap_or_else(|| vec!["*".to_string()]);

        // Evaluate each action against each resource
        let mut results = Vec::new();

        for action in &request.action_names {
            for resource in &resources {
                let decision = self.evaluate_action(&policies, action, resource);
                let matched_statements = self.find_matching_statements(&policies, action, resource);

                results.push(EvaluationResult {
                    eval_action_name: action.clone(),
                    eval_resource_name: resource.clone(),
                    eval_decision: decision,
                    matched_statements,
                    missing_context_values: vec![], // TODO: Context evaluation
                });
            }
        }

        Ok(SimulatePolicyResponse {
            evaluation_results: results,
            is_truncated: false,
        })
    }

    /// Simulate a principal's (user or role) effective policies
    ///
    /// Fetches the principal's attached policies from the store and evaluates them.
    pub async fn simulate_principal_policy(
        &self,
        request: SimulatePrincipalPolicyRequest,
    ) -> Result<SimulatePolicyResponse> {
        // Parse principal ARN to determine type
        let (principal_type, principal_name) =
            self.parse_principal_arn(&request.policy_source_arn)?;

        // Fetch principal's policies from store
        let mut policies = self
            .fetch_principal_policies(&principal_type, &principal_name)
            .await?;

        // Fetch permissions boundary if present
        let boundary = self
            .fetch_permissions_boundary(&principal_type, &principal_name)
            .await?;

        // Add additional policy documents from request if provided
        if let Some(extra_policies) = request.policy_input_list {
            for policy_str in extra_policies {
                let policy: PolicyDocument =
                    serde_json::from_str(&policy_str).map_err(|e| AmiError::InvalidParameter {
                        message: format!("Invalid policy document: {}", e),
                    })?;
                policies.push(policy);
            }
        }

        // Default resource if not provided
        let resources = request
            .resource_arns
            .unwrap_or_else(|| vec!["*".to_string()]);

        // Evaluate each action against each resource
        let mut results = Vec::new();

        for action in &request.action_names {
            for resource in &resources {
                // Use boundary-aware evaluation if boundary exists
                let decision = self.evaluate_action_with_boundary(
                    &policies,
                    action,
                    resource,
                    boundary.as_ref(),
                );
                let matched_statements = self.find_matching_statements(&policies, action, resource);

                results.push(EvaluationResult {
                    eval_action_name: action.clone(),
                    eval_resource_name: resource.clone(),
                    eval_decision: decision,
                    matched_statements,
                    missing_context_values: vec![], // TODO: Context evaluation
                });
            }
        }

        Ok(SimulatePolicyResponse {
            evaluation_results: results,
            is_truncated: false,
        })
    }

    // Helper methods

    /// Parse principal ARN to extract type and name
    fn parse_principal_arn(&self, arn: &str) -> Result<(String, String)> {
        // Expected formats:
        // arn:aws:iam::123456789012:user/alice
        // arn:aws:iam::123456789012:user/path/to/alice
        // arn:aws:iam::123456789012:role/MyRole
        // arn:aws:iam::123456789012:role/path/MyRole

        let parts: Vec<&str> = arn.split(':').collect();
        if parts.len() < 6 {
            return Err(AmiError::InvalidParameter {
                message: format!("Invalid principal ARN: {}", arn),
            });
        }

        let resource_part = parts[5]; // "user/alice" or "user/path/alice"
        let resource_parts: Vec<&str> = resource_part.split('/').collect();

        if resource_parts.len() < 2 {
            return Err(AmiError::InvalidParameter {
                message: format!("Invalid principal ARN format: {}", arn),
            });
        }

        let principal_type = resource_parts[0].to_string();
        // The principal name is always the last part (after the type and any path components)
        let principal_name = resource_parts[resource_parts.len() - 1].to_string();

        Ok((principal_type, principal_name))
    }

    /// Fetch policies for a user or role
    async fn fetch_principal_policies(
        &self,
        principal_type: &str,
        principal_name: &str,
    ) -> Result<Vec<PolicyDocument>> {
        let policies = Vec::new();

        match principal_type {
            "user" => {
                // Verify user exists
                let _user = self
                    .store
                    .read()
                    .unwrap()
                    .get_user(principal_name)
                    .await?
                    .ok_or_else(|| AmiError::ResourceNotFound {
                        resource: format!("User: {}", principal_name),
                    })?;

                // TODO: Policy attachments are not yet implemented in the User model
                // In a full implementation, we would:
                // 1. Query a policy_attachments table/map
                // 2. Fetch all attached policies
                // 3. Include inline policies if any
                // For now, return empty list (will use policy_input_list from request instead)
            }
            "role" => {
                // Verify role exists
                let _role = self
                    .store
                    .read()
                    .unwrap()
                    .get_role(principal_name)
                    .await?
                    .ok_or_else(|| AmiError::ResourceNotFound {
                        resource: format!("Role: {}", principal_name),
                    })?;

                // TODO: Same as user - policy attachments need separate tracking
            }
            _ => {
                return Err(AmiError::InvalidParameter {
                    message: format!("Unsupported principal type: {}", principal_type),
                })
            }
        }

        Ok(policies)
    }

    /// Fetch permissions boundary for a user or role
    async fn fetch_permissions_boundary(
        &self,
        principal_type: &str,
        principal_name: &str,
    ) -> Result<Option<crate::wami::policies::Policy>> {
        let boundary_arn = match principal_type {
            "user" => {
                let user = self
                    .store
                    .read()
                    .unwrap()
                    .get_user(principal_name)
                    .await?
                    .ok_or_else(|| AmiError::ResourceNotFound {
                        resource: format!("User: {}", principal_name),
                    })?;
                user.permissions_boundary
            }
            "role" => {
                let role = self
                    .store
                    .read()
                    .unwrap()
                    .get_role(principal_name)
                    .await?
                    .ok_or_else(|| AmiError::ResourceNotFound {
                        resource: format!("Role: {}", principal_name),
                    })?;
                role.permissions_boundary
            }
            _ => {
                return Err(AmiError::InvalidParameter {
                    message: format!("Unsupported principal type: {}", principal_type),
                })
            }
        };

        // Fetch the boundary policy if it exists
        if let Some(arn) = boundary_arn {
            let policy = self.store.read().unwrap().get_policy(&arn).await?;
            Ok(policy)
        } else {
            Ok(None)
        }
    }

    /// Evaluate a single action/resource combination against policies
    fn evaluate_action(&self, policies: &[PolicyDocument], action: &str, resource: &str) -> String {
        let mut has_allow = false;
        let mut has_deny = false;

        for policy in policies {
            for statement in &policy.statement {
                let action_matches = statement
                    .action
                    .iter()
                    .any(|a| Self::matches_pattern(action, a));

                let resource_matches = statement
                    .resource
                    .iter()
                    .any(|r| Self::matches_pattern(resource, r));

                if action_matches && resource_matches {
                    if statement.effect == "Deny" {
                        has_deny = true;
                    } else if statement.effect == "Allow" {
                        has_allow = true;
                    }
                }
            }
        }

        // Explicit deny always wins
        if has_deny {
            "denied".to_string()
        } else if has_allow {
            "allowed".to_string()
        } else {
            "implicitDeny".to_string()
        }
    }

    /// Evaluate action with permissions boundary
    ///
    /// The effective permissions are the intersection of:
    /// 1. Identity-based policies (must allow)
    /// 2. Permissions boundary (must allow)
    ///
    /// If either denies, the final result is deny.
    fn evaluate_action_with_boundary(
        &self,
        policies: &[PolicyDocument],
        action: &str,
        resource: &str,
        boundary: Option<&crate::wami::policies::Policy>,
    ) -> String {
        // Step 1: Check explicit deny in identity policies
        for policy in policies {
            for statement in &policy.statement {
                let action_matches = statement
                    .action
                    .iter()
                    .any(|a| Self::matches_pattern(action, a));

                let resource_matches = statement
                    .resource
                    .iter()
                    .any(|r| Self::matches_pattern(resource, r));

                if action_matches && resource_matches && statement.effect == "Deny" {
                    return "denied".to_string();
                }
            }
        }

        // Step 2: Check if identity policies allow
        let identity_allows = policies.iter().any(|policy| {
            policy.statement.iter().any(|statement| {
                let action_matches = statement
                    .action
                    .iter()
                    .any(|a| Self::matches_pattern(action, a));
                let resource_matches = statement
                    .resource
                    .iter()
                    .any(|r| Self::matches_pattern(resource, r));

                action_matches && resource_matches && statement.effect == "Allow"
            })
        });

        if !identity_allows {
            return "implicitDeny".to_string();
        }

        // Step 3: Check permissions boundary (if present)
        if let Some(boundary_policy) = boundary {
            match crate::wami::policies::permissions_boundary::operations::is_allowed_by_boundary(
                action,
                resource,
                boundary_policy,
            ) {
                Ok(allowed) => {
                    if !allowed {
                        return "denied".to_string(); // Boundary restricts the action
                    }
                }
                Err(_) => {
                    // If boundary evaluation fails, deny for safety
                    return "denied".to_string();
                }
            }
        }

        // Both identity policies and boundary allow
        "allowed".to_string()
    }

    /// Find all statements that match the action/resource
    fn find_matching_statements(
        &self,
        policies: &[PolicyDocument],
        action: &str,
        resource: &str,
    ) -> Vec<StatementMatch> {
        let mut matches = Vec::new();

        for policy in policies {
            for statement in &policy.statement {
                let action_matches = statement
                    .action
                    .iter()
                    .any(|a| Self::matches_pattern(action, a));

                let resource_matches = statement
                    .resource
                    .iter()
                    .any(|r| Self::matches_pattern(resource, r));

                if action_matches || resource_matches {
                    matches.push(StatementMatch {
                        source_policy_id: None, // PolicyStatement doesn't have sid field
                        effect: statement.effect.clone(),
                        matched_action: action_matches,
                        matched_resource: resource_matches,
                    });
                }
            }
        }

        matches
    }

    /// Check if a value matches a pattern (with wildcard support)
    fn matches_pattern(value: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            return value.starts_with(prefix);
        }

        value == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::identity::user::builder::build_user;

    fn setup_service() -> EvaluationService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        EvaluationService::new(store, "123456789012".to_string())
    }

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:.*:12345678:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_simulate_custom_policy_allow() {
        let service = setup_service();

        let policy_doc = r#"{
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": "s3:GetObject",
                    "Resource": "arn:aws:s3:::mybucket/*"
                }
            ]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy_doc.to_string()],
            action_names: vec!["s3:GetObject".to_string()],
            resource_arns: Some(vec!["arn:aws:s3:::mybucket/file.txt".to_string()]),
            context_entries: None,
        };

        let response = service.simulate_custom_policy(request).await.unwrap();

        assert_eq!(response.evaluation_results.len(), 1);
        assert_eq!(response.evaluation_results[0].eval_decision, "allowed");
    }

    #[tokio::test]
    async fn test_simulate_custom_policy_deny() {
        let service = setup_service();

        let policy_doc = r#"{
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Deny",
                    "Action": "s3:DeleteObject",
                    "Resource": "*"
                }
            ]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy_doc.to_string()],
            action_names: vec!["s3:DeleteObject".to_string()],
            resource_arns: Some(vec!["arn:aws:s3:::mybucket/file.txt".to_string()]),
            context_entries: None,
        };

        let response = service.simulate_custom_policy(request).await.unwrap();

        assert_eq!(response.evaluation_results.len(), 1);
        assert_eq!(response.evaluation_results[0].eval_decision, "denied");
    }

    #[tokio::test]
    async fn test_simulate_custom_policy_implicit_deny() {
        let service = setup_service();

        let policy_doc = r#"{
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": "s3:GetObject",
                    "Resource": "arn:aws:s3:::mybucket/*"
                }
            ]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy_doc.to_string()],
            action_names: vec!["s3:PutObject".to_string()],
            resource_arns: Some(vec!["arn:aws:s3:::mybucket/file.txt".to_string()]),
            context_entries: None,
        };

        let response = service.simulate_custom_policy(request).await.unwrap();

        assert_eq!(response.evaluation_results.len(), 1);
        assert_eq!(response.evaluation_results[0].eval_decision, "implicitDeny");
    }

    #[tokio::test]
    async fn test_simulate_custom_policy_wildcard() {
        let service = setup_service();

        let policy_doc = r#"{
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": "s3:*",
                    "Resource": "*"
                }
            ]
        }"#;

        let request = SimulateCustomPolicyRequest {
            policy_input_list: vec![policy_doc.to_string()],
            action_names: vec!["s3:GetObject".to_string(), "s3:PutObject".to_string()],
            resource_arns: Some(vec!["arn:aws:s3:::anybucket/anyfile".to_string()]),
            context_entries: None,
        };

        let response = service.simulate_custom_policy(request).await.unwrap();

        assert_eq!(response.evaluation_results.len(), 2);
        assert!(response
            .evaluation_results
            .iter()
            .all(|r| r.eval_decision == "allowed"));
    }

    #[tokio::test]
    async fn test_simulate_principal_policy_user() {
        let service = setup_service();
        let context = test_context();

        // Create a user
        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();

        service
            .store
            .write()
            .unwrap()
            .create_user(user)
            .await
            .unwrap();

        // Create a policy document for testing
        let policy_doc = r#"{
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": "ec2:DescribeInstances",
                    "Resource": "*"
                }
            ]
        }"#;

        // Note: Since User model doesn't have attached_policies yet,
        // we pass the policy via policy_input_list
        let request = SimulatePrincipalPolicyRequest {
            policy_source_arn: "arn:aws:iam::123456789012:user/alice".to_string(),
            action_names: vec!["ec2:DescribeInstances".to_string()],
            resource_arns: None,
            policy_input_list: Some(vec![policy_doc.to_string()]),
            context_entries: None,
        };

        let response = service.simulate_principal_policy(request).await.unwrap();

        assert_eq!(response.evaluation_results.len(), 1);
        assert_eq!(response.evaluation_results[0].eval_decision, "allowed");
    }

    #[tokio::test]
    async fn test_parse_principal_arn_user() {
        let service = setup_service();

        let (principal_type, principal_name) = service
            .parse_principal_arn("arn:aws:iam::123456789012:user/alice")
            .unwrap();

        assert_eq!(principal_type, "user");
        assert_eq!(principal_name, "alice");
    }

    #[tokio::test]
    async fn test_parse_principal_arn_role() {
        let service = setup_service();

        let (principal_type, principal_name) = service
            .parse_principal_arn("arn:aws:iam::123456789012:role/MyRole")
            .unwrap();

        assert_eq!(principal_type, "role");
        assert_eq!(principal_name, "MyRole");
    }

    #[tokio::test]
    async fn test_parse_principal_arn_with_path() {
        let service = setup_service();

        let (principal_type, principal_name) = service
            .parse_principal_arn("arn:aws:iam::123456789012:user/department/team/alice")
            .unwrap();

        assert_eq!(principal_type, "user");
        // Principal name is just the name, not the path
        // Path is /department/team/ and name is alice
        assert_eq!(principal_name, "alice");
    }
}
