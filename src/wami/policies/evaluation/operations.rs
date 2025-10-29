//! Policy Evaluation Domain Operations
//!
//! Pure business logic functions for policy evaluation and simulation.

use crate::error::{AmiError, Result};
use crate::types::PolicyDocument;

/// Pure domain operations for policy evaluation
pub mod policy_evaluation_operations {
    use super::*;

    /// Evaluate if an action is allowed by a policy (pure function)
    pub fn is_action_allowed(
        policy_doc: &PolicyDocument,
        action: &str,
        resource: &str,
    ) -> bool {
        for statement in &policy_doc.statement {
            // Check if action matches
            let action_matches = statement
                .action
                .iter()
                .any(|a| action_matches_pattern(action, a));

            // Check if resource matches
            let resource_matches = statement
                .resource
                .iter()
                .any(|r| resource_matches_pattern(resource, r));

            if action_matches && resource_matches {
                return statement.effect == "Allow";
            }
        }

        false // Default deny
    }

    /// Check if an action matches a pattern (with wildcards) (pure function)
    fn action_matches_pattern(action: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return action.starts_with(prefix);
        }

        action == pattern
    }

    /// Check if a resource matches a pattern (with wildcards) (pure function)
    fn resource_matches_pattern(resource: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return resource.starts_with(prefix);
        }

        resource == pattern
    }

    /// Evaluate multiple policies (pure function)
    pub fn evaluate_policies(
        policies: &[PolicyDocument],
        action: &str,
        resource: &str,
    ) -> EvaluationResult {
        let mut has_allow = false;
        let mut has_deny = false;

        for policy in policies {
            for statement in &policy.statement {
                let action_matches = statement
                    .action
                    .iter()
                    .any(|a| action_matches_pattern(action, a));

                let resource_matches = statement
                    .resource
                    .iter()
                    .any(|r| resource_matches_pattern(resource, r));

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
            EvaluationResult::Deny
        } else if has_allow {
            EvaluationResult::Allow
        } else {
            EvaluationResult::ImplicitDeny
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationResult {
    Allow,
    Deny,
    ImplicitDeny,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PolicyStatement;

    #[test]
    fn test_action_allowed() {
        let policy = PolicyDocument {
            version: Some("2012-10-17".to_string()),
            statement: vec![PolicyStatement {
                sid: None,
                effect: "Allow".to_string(),
                action: vec!["s3:GetObject".to_string()],
                resource: vec!["arn:aws:s3:::bucket/*".to_string()],
                principal: None,
                condition: None,
            }],
        };

        assert!(policy_evaluation_operations::is_action_allowed(
            &policy,
            "s3:GetObject",
            "arn:aws:s3:::bucket/key"
        ));

        assert!(!policy_evaluation_operations::is_action_allowed(
            &policy,
            "s3:PutObject",
            "arn:aws:s3:::bucket/key"
        ));
    }

    #[test]
    fn test_wildcard_action() {
        let policy = PolicyDocument {
            version: Some("2012-10-17".to_string()),
            statement: vec![PolicyStatement {
                sid: None,
                effect: "Allow".to_string(),
                action: vec!["s3:*".to_string()],
                resource: vec!["*".to_string()],
                principal: None,
                condition: None,
            }],
        };

        assert!(policy_evaluation_operations::is_action_allowed(
            &policy,
            "s3:GetObject",
            "arn:aws:s3:::bucket/key"
        ));

        assert!(policy_evaluation_operations::is_action_allowed(
            &policy,
            "s3:PutObject",
            "arn:aws:s3:::bucket/key"
        ));
    }

    #[test]
    fn test_explicit_deny() {
        let policies = vec![
            PolicyDocument {
                version: Some("2012-10-17".to_string()),
                statement: vec![PolicyStatement {
                    sid: None,
                    effect: "Allow".to_string(),
                    action: vec!["s3:*".to_string()],
                    resource: vec!["*".to_string()],
                    principal: None,
                    condition: None,
                }],
            },
            PolicyDocument {
                version: Some("2012-10-17".to_string()),
                statement: vec![PolicyStatement {
                    sid: None,
                    effect: "Deny".to_string(),
                    action: vec!["s3:DeleteObject".to_string()],
                    resource: vec!["*".to_string()],
                    principal: None,
                    condition: None,
                }],
            },
        ];

        let result = policy_evaluation_operations::evaluate_policies(
            &policies,
            "s3:DeleteObject",
            "arn:aws:s3:::bucket/key",
        );

        assert_eq!(result, EvaluationResult::Deny);
    }
}
