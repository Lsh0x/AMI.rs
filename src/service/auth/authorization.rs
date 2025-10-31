//! Authorization Service - Permission checking and policy evaluation
//!
//! This service handles authorization checks:
//! 1. Root users bypass all checks (full access)
//! 2. Regular users are subject to policy evaluation
//! 3. Policies are evaluated from user, groups, and roles
//! 4. Deny overrides Allow
//!
//! # Example
//!
//! ```rust,no_run
//! use wami::{AuthorizationService, WamiContext, store::memory::InMemoryWamiStore, WamiArn};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
//!     let authz_service = AuthorizationService::new(store);
//!
//!     // Assume we have an authenticated context
//!     let context = todo!("Get from authentication");
//!     let resource_arn: WamiArn = "arn:wami:iam:12345678:wami:999:user/alice".parse()?;
//!
//!     // Check if user can perform action
//!     let allowed = authz_service
//!         .authorize(&context, "iam:GetUser", &resource_arn)
//!         .await?;
//!
//!     if allowed {
//!         println!("Access granted");
//!     } else {
//!         println!("Access denied");
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::arn::WamiArn;
use crate::context::WamiContext;
use crate::error::{AmiError, Result};
use crate::store::traits::{GroupStore, PolicyStore, RoleStore, UserStore};
use crate::types::PolicyDocument;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Authorization Service
///
/// Handles permission checking based on IAM policies.
pub struct AuthorizationService<S>
where
    S: UserStore + GroupStore + RoleStore + PolicyStore + Send + Sync,
{
    store: Arc<RwLock<S>>,
}

impl<S> AuthorizationService<S>
where
    S: UserStore + GroupStore + RoleStore + PolicyStore + Send + Sync,
{
    /// Create a new authorization service
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Authorize an action on a resource
    ///
    /// This is the main authorization method. It checks if the caller
    /// (from the context) is allowed to perform the specified action
    /// on the target resource.
    ///
    /// # Arguments
    ///
    /// * `context` - The authenticated context (contains caller info)
    /// * `action` - The action to perform (e.g., "iam:GetUser")
    /// * `resource_arn` - The target resource ARN
    ///
    /// # Returns
    ///
    /// `true` if the action is allowed, `false` otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if there's a problem accessing the store or
    /// evaluating policies.
    pub async fn authorize(
        &self,
        context: &WamiContext,
        action: &str,
        resource_arn: &WamiArn,
    ) -> Result<bool> {
        // Root users bypass all authorization checks
        if context.is_root() {
            return Ok(true);
        }

        // Extract user name from caller ARN
        let user_name = self.extract_user_name_from_arn(context.caller_arn())?;

        // Evaluate policies for this user
        self.evaluate_user_policies(&user_name, action, resource_arn)
            .await
    }

    /// Check if access is denied (returns an error if not authorized)
    ///
    /// This is a convenience method that throws an `AccessDenied` error
    /// if the authorization check fails.
    pub async fn check_or_deny(
        &self,
        context: &WamiContext,
        action: &str,
        resource_arn: &WamiArn,
    ) -> Result<()> {
        let allowed = self.authorize(context, action, resource_arn).await?;

        if !allowed {
            return Err(AmiError::AccessDenied {
                message: format!(
                    "User {} is not authorized to perform {} on {}",
                    context.caller_arn(),
                    action,
                    resource_arn
                ),
            });
        }

        Ok(())
    }

    /// Evaluate all policies for a user
    ///
    /// This includes:
    /// - User's attached managed policies
    /// - User's inline policies
    /// - Policies from user's groups
    /// - TODO: Assumed role policies
    async fn evaluate_user_policies(
        &self,
        user_name: &str,
        action: &str,
        resource_arn: &WamiArn,
    ) -> Result<bool> {
        let store = self.store.read().await;

        // Get user's attached managed policies
        let attached_policies = store.list_attached_user_policies(user_name).await?;

        // Check each attached policy
        for policy_arn in attached_policies {
            // Get the policy document
            if let Some(policy) = store.get_policy(&policy_arn).await? {
                let policy_doc: PolicyDocument = serde_json::from_str(&policy.policy_document)
                    .unwrap_or_else(|_| PolicyDocument {
                        version: "2012-10-17".to_string(),
                        statement: vec![],
                    });

                // Evaluate the policy
                match self.evaluate_policy_document(&policy_doc, action, resource_arn) {
                    PolicyEffect::Allow => return Ok(true),
                    PolicyEffect::Deny => return Ok(false),
                    PolicyEffect::NoMatch => continue,
                }
            }
        }

        // Get user's inline policies
        let inline_policies = store.list_user_policies(user_name).await?;

        for policy_name in inline_policies {
            if let Some(policy_doc_str) = store.get_user_policy(user_name, &policy_name).await? {
                let policy_doc: PolicyDocument = serde_json::from_str(&policy_doc_str)
                    .unwrap_or_else(|_| PolicyDocument {
                        version: "2012-10-17".to_string(),
                        statement: vec![],
                    });

                match self.evaluate_policy_document(&policy_doc, action, resource_arn) {
                    PolicyEffect::Allow => return Ok(true),
                    PolicyEffect::Deny => return Ok(false),
                    PolicyEffect::NoMatch => continue,
                }
            }
        }

        // TODO: Get policies from user's groups
        // TODO: Get policies from assumed roles

        // Default deny - if no policy explicitly allows, deny
        Ok(false)
    }

    /// Evaluate a single policy document
    ///
    /// Returns:
    /// - `Allow` if the policy explicitly allows the action
    /// - `Deny` if the policy explicitly denies the action (deny overrides allow)
    /// - `NoMatch` if the policy doesn't apply to this action/resource
    fn evaluate_policy_document(
        &self,
        policy: &PolicyDocument,
        action: &str,
        resource_arn: &WamiArn,
    ) -> PolicyEffect {
        let resource_str = resource_arn.to_string();

        // First check for explicit denies (deny overrides allow)
        for statement in &policy.statement {
            if statement.effect.to_lowercase() == "deny"
                && self.matches_action(&statement.action, action)
                && self.matches_resource(&statement.resource, &resource_str)
            {
                return PolicyEffect::Deny;
            }
        }

        // Then check for allows
        for statement in &policy.statement {
            if statement.effect.to_lowercase() == "allow"
                && self.matches_action(&statement.action, action)
                && self.matches_resource(&statement.resource, &resource_str)
            {
                return PolicyEffect::Allow;
            }
        }

        PolicyEffect::NoMatch
    }

    /// Check if an action matches the policy statement
    ///
    /// Supports wildcards: `iam:*`, `*`
    fn matches_action(&self, policy_actions: &[String], action: &str) -> bool {
        for policy_action in policy_actions {
            if policy_action == "*" {
                return true;
            }

            if policy_action == action {
                return true;
            }

            // Check wildcard patterns like "iam:*"
            if policy_action.ends_with("*") {
                let prefix = &policy_action[..policy_action.len() - 1];
                if action.starts_with(prefix) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a resource matches the policy statement
    ///
    /// Supports wildcards: `*`, `arn:wami:iam:*:user/*`
    fn matches_resource(&self, policy_resources: &[String], resource: &str) -> bool {
        for policy_resource in policy_resources {
            if policy_resource == "*" {
                return true;
            }

            if policy_resource == resource {
                return true;
            }

            // Check wildcard patterns
            if policy_resource.contains('*') && self.wildcard_match(policy_resource, resource) {
                return true;
            }
        }

        false
    }

    /// Simple wildcard matching (supports * wildcards)
    fn wildcard_match(&self, pattern: &str, text: &str) -> bool {
        let parts: Vec<&str> = pattern.split('*').collect();

        if parts.is_empty() {
            return false;
        }

        let mut text_pos = 0;

        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            // For the first part, it must match at the beginning
            if i == 0 && !text.starts_with(part) {
                return false;
            }

            // For the last part, it must match at the end
            if i == parts.len() - 1 && !text.ends_with(part) {
                return false;
            }

            // Find the part in the remaining text
            if let Some(pos) = text[text_pos..].find(part) {
                text_pos += pos + part.len();
            } else {
                return false;
            }
        }

        true
    }

    /// Extract user name from a user ARN
    fn extract_user_name_from_arn(&self, arn: &WamiArn) -> Result<String> {
        // Check if this is a user resource
        if arn.resource.resource_type == "user" {
            // Return the resource ID as user_name
            // Note: resource_id is the stable user ID, not necessarily the user name
            // This might need adjustment based on how user_name is mapped
            Ok(arn.resource.resource_id.clone())
        } else {
            Err(AmiError::InvalidParameter {
                message: "Caller ARN is not a user ARN".to_string(),
            })
        }
    }
}

/// Policy evaluation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PolicyEffect {
    /// Policy explicitly allows the action
    Allow,
    /// Policy explicitly denies the action (overrides any allow)
    Deny,
    /// Policy doesn't match this action/resource
    NoMatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use crate::types::{PolicyDocument, PolicyStatement};

    #[test]
    fn test_matches_action() {
        let store = InMemoryWamiStore::new();
        let service = AuthorizationService {
            store: Arc::new(RwLock::new(store)),
        };

        // Exact match
        assert!(service.matches_action(&["iam:GetUser".to_string()], "iam:GetUser"));

        // Wildcard all
        assert!(service.matches_action(&["*".to_string()], "iam:GetUser"));

        // Wildcard prefix
        assert!(service.matches_action(&["iam:*".to_string()], "iam:GetUser"));
        assert!(service.matches_action(&["iam:*".to_string()], "iam:CreateUser"));

        // No match
        assert!(!service.matches_action(&["s3:GetObject".to_string()], "iam:GetUser"));
    }

    #[test]
    fn test_matches_resource() {
        let store = InMemoryWamiStore::new();
        let service = AuthorizationService {
            store: Arc::new(RwLock::new(store)),
        };

        // Exact match
        assert!(service.matches_resource(
            &["arn:wami:iam:12345678:wami:999:user/alice".to_string()],
            "arn:wami:iam:12345678:wami:999:user/alice"
        ));

        // Wildcard all
        assert!(service.matches_resource(
            &["*".to_string()],
            "arn:wami:iam:12345678:wami:999:user/alice"
        ));

        // Wildcard pattern
        assert!(service.matches_resource(
            &["arn:wami:iam:*:user/*".to_string()],
            "arn:wami:iam:12345678:wami:999:user/alice"
        ));

        // No match
        assert!(!service.matches_resource(
            &["arn:wami:iam:12345678:wami:999:role/*".to_string()],
            "arn:wami:iam:12345678:wami:999:user/alice"
        ));
    }

    #[test]
    fn test_wildcard_match() {
        let store = InMemoryWamiStore::new();
        let service = AuthorizationService {
            store: Arc::new(RwLock::new(store)),
        };

        assert!(service.wildcard_match("arn:*:user/*", "arn:wami:iam:12345678:wami:999:user/alice"));
        assert!(service.wildcard_match("*.example.com", "api.example.com"));
        assert!(service.wildcard_match("test-*-prod", "test-api-prod"));

        assert!(
            !service.wildcard_match("arn:*:role/*", "arn:wami:iam:12345678:wami:999:user/alice")
        );
    }

    #[test]
    fn test_matches_action_edge_cases() {
        let store = InMemoryWamiStore::new();
        let service = AuthorizationService {
            store: Arc::new(RwLock::new(store)),
        };

        // Empty actions
        assert!(!service.matches_action(&[], "iam:GetUser"));

        // Multiple wildcards
        assert!(service.matches_action(&["iam:*".to_string(), "s3:*".to_string()], "iam:GetUser"));

        // Exact match in list
        assert!(service.matches_action(
            &["s3:GetObject".to_string(), "iam:GetUser".to_string()],
            "iam:GetUser"
        ));

        // No match
        assert!(!service.matches_action(&["s3:GetObject".to_string()], "iam:GetUser"));

        // Wildcard at end
        assert!(service.matches_action(&["iam:Get*".to_string()], "iam:GetUser"));
    }

    #[test]
    fn test_matches_resource_edge_cases() {
        let store = InMemoryWamiStore::new();
        let service = AuthorizationService {
            store: Arc::new(RwLock::new(store)),
        };

        // Empty resources
        assert!(!service.matches_resource(&[], "arn:wami:iam:12345678:wami:999:user/alice"));

        // Multiple patterns
        assert!(service.matches_resource(
            &[
                "arn:wami:iam:*:role/*".to_string(),
                "arn:wami:iam:*:user/*".to_string()
            ],
            "arn:wami:iam:12345678:wami:999:user/alice"
        ));

        // Complex wildcard pattern
        assert!(service.matches_resource(
            &["arn:wami:iam:t*:wami:*:user/al*".to_string()],
            "arn:wami:iam:12345678:wami:999:user/alice"
        ));
    }

    #[test]
    fn test_evaluate_policy_deny_overrides_allow() {
        let store = InMemoryWamiStore::new();
        let service = AuthorizationService {
            store: Arc::new(RwLock::new(store)),
        };

        let policy = PolicyDocument {
            version: "2012-10-17".to_string(),
            statement: vec![
                PolicyStatement {
                    effect: "Allow".to_string(),
                    action: vec!["iam:*".to_string()],
                    resource: vec!["*".to_string()],
                    condition: None,
                },
                PolicyStatement {
                    effect: "Deny".to_string(),
                    action: vec!["iam:DeleteUser".to_string()],
                    resource: vec!["*".to_string()],
                    condition: None,
                },
            ],
        };

        let resource: WamiArn = "arn:wami:iam:12345678:wami:999:user/alice".parse().unwrap();
        let effect = service.evaluate_policy_document(&policy, "iam:DeleteUser", &resource);

        // Deny should override Allow
        assert_eq!(effect, PolicyEffect::Deny);
    }

    #[test]
    fn test_evaluate_policy_no_match() {
        let store = InMemoryWamiStore::new();
        let service = AuthorizationService {
            store: Arc::new(RwLock::new(store)),
        };

        let policy = PolicyDocument {
            version: "2012-10-17".to_string(),
            statement: vec![PolicyStatement {
                effect: "Allow".to_string(),
                action: vec!["s3:GetObject".to_string()],
                resource: vec!["*".to_string()],
                condition: None,
            }],
        };

        let resource: WamiArn = "arn:wami:iam:12345678:wami:999:user/alice".parse().unwrap();
        let effect = service.evaluate_policy_document(&policy, "iam:GetUser", &resource);

        assert_eq!(effect, PolicyEffect::NoMatch);
    }

    #[test]
    fn test_evaluate_policy_case_insensitive_effect() {
        let store = InMemoryWamiStore::new();
        let service = AuthorizationService {
            store: Arc::new(RwLock::new(store)),
        };

        let policy = PolicyDocument {
            version: "2012-10-17".to_string(),
            statement: vec![PolicyStatement {
                effect: "DENY".to_string(), // Uppercase
                action: vec!["iam:GetUser".to_string()],
                resource: vec!["*".to_string()],
                condition: None,
            }],
        };

        let resource: WamiArn = "arn:wami:iam:12345678:wami:999:user/alice".parse().unwrap();
        let effect = service.evaluate_policy_document(&policy, "iam:GetUser", &resource);

        assert_eq!(effect, PolicyEffect::Deny);
    }
}
