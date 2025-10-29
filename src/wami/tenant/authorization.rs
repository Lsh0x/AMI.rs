//! Tenant Authorization using IAM Policy Evaluation
//!
//! This module provides tenant authorization using the IAM policy evaluation system.
//! It can work both with and without a store, making it flexible and reusable.
//!
//! # Architecture
//!
//! Instead of maintaining a separate authorization system, we use IAM policies to control
//! tenant operations. This provides:
//! - Unified authorization model across IAM and Tenant operations
//! - Fine-grained permissions using standard IAM policy syntax
//! - Flexibility to work with or without persistent storage
//!
//! # Example without Store (Standalone)
//!
//! ```rust
//! use wami::tenant::authorization::{TenantAuthorizer, TenantAction};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an authorizer with inline policies
//! let policies = vec![
//!     r#"{
//!         "Version": "2012-10-17",
//!         "Statement": [{
//!             "Effect": "Allow",
//!             "Action": ["tenant:Read", "tenant:Update"],
//!             "Resource": "arn:wami:tenant::acme/*"
//!         }]
//!     }"#.to_string(),
//! ];
//!
//! let authorizer = TenantAuthorizer::new(policies);
//!
//! // Check permissions
//! let allowed = authorizer.check_permission(
//!     "arn:aws:iam::123456789012:user/alice",
//!     "acme/engineering",
//!     TenantAction::Read,
//! ).await?;
//!
//! assert!(allowed);
//! # Ok(())
//! # }
//! ```
//!
//! # Example with Store
//!
//! ```rust
//! use wami::wami::tenant::authorization::{TenantAuthorizer, TenantAction};
//! use wami::store::memory::InMemoryWamiStore;
//! use wami::store::traits::PolicyStore;
//! use wami::provider::AwsProvider;
//! use wami::wami::policies::policy::builder::build_policy;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut store = InMemoryWamiStore::default();
//! let provider = AwsProvider::new();
//!
//! // Create a policy in the store
//! let policy = build_policy(
//!     "TenantAdminPolicy".to_string(),
//!     r#"{
//!         "Version": "2012-10-17",
//!         "Statement": [{
//!             "Effect": "Allow",
//!             "Action": "tenant:*",
//!             "Resource": "arn:wami:tenant::acme/*"
//!         }]
//!     }"#.to_string(),
//!     Some("/".to_string()),
//!     None, // description
//!     None, // tags
//!     &provider,
//!     "123456789012",
//! );
//!
//! let created_policy = store.create_policy(policy).await?;
//!
//! // Use the policy for authorization
//! let authorizer = TenantAuthorizer::new(vec![created_policy.policy_document]);
//!
//! let allowed = authorizer.check_permission(
//!     "arn:aws:iam::123456789012:user/admin",
//!     "acme/engineering",
//!     TenantAction::Delete,
//! ).await?;
//!
//! assert!(allowed);
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::store::traits::TenantStore;
use crate::store::Store;
use crate::types::{PolicyDocument, PolicyStatement};
use crate::wami::tenant::TenantId;

/// Tenant actions that can be authorized
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantAction {
    /// Read tenant information (tenant:Read)
    Read,
    /// Update tenant (tenant:Update)
    Update,
    /// Delete tenant (tenant:Delete)
    Delete,
    /// Create sub-tenant (tenant:CreateSubTenant)
    CreateSubTenant,
    /// Manage users in tenant (tenant:ManageUsers)
    ManageUsers,
    /// Manage roles in tenant (tenant:ManageRoles)
    ManageRoles,
    /// Manage policies in tenant (tenant:ManagePolicies)
    ManagePolicies,
    /// All tenant actions (tenant:*)
    All,
}

impl TenantAction {
    /// Convert action to IAM action string
    pub fn to_action_string(&self) -> &'static str {
        match self {
            TenantAction::Read => "tenant:Read",
            TenantAction::Update => "tenant:Update",
            TenantAction::Delete => "tenant:Delete",
            TenantAction::CreateSubTenant => "tenant:CreateSubTenant",
            TenantAction::ManageUsers => "tenant:ManageUsers",
            TenantAction::ManageRoles => "tenant:ManageRoles",
            TenantAction::ManagePolicies => "tenant:ManagePolicies",
            TenantAction::All => "tenant:*",
        }
    }
}

/// Tenant Authorizer using IAM policies
///
/// This authorizer evaluates tenant permissions using standard IAM policy documents.
/// It can work standalone (without a store) by accepting policies directly.
pub struct TenantAuthorizer {
    /// Policy documents to evaluate
    policies: Vec<PolicyDocument>,
}

impl TenantAuthorizer {
    /// Create a new authorizer with the given policy documents
    ///
    /// # Arguments
    ///
    /// * `policy_json_list` - List of IAM policy documents as JSON strings
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::tenant::authorization::TenantAuthorizer;
    ///
    /// let policies = vec![
    ///     r#"{
    ///         "Version": "2012-10-17",
    ///         "Statement": [{
    ///             "Effect": "Allow",
    ///             "Action": "tenant:*",
    ///             "Resource": "*"
    ///         }]
    ///     }"#.to_string(),
    /// ];
    ///
    /// let authorizer = TenantAuthorizer::new(policies);
    /// ```
    pub fn new(policy_json_list: Vec<String>) -> Self {
        let mut policies = Vec::new();
        for policy_json in policy_json_list {
            if let Ok(doc) = serde_json::from_str::<PolicyDocument>(&policy_json) {
                policies.push(doc);
            }
        }
        Self { policies }
    }

    /// Create an authorizer with already-parsed policy documents
    pub fn from_documents(policies: Vec<PolicyDocument>) -> Self {
        Self { policies }
    }

    /// Check if a principal has permission to perform an action on a tenant
    ///
    /// # Arguments
    ///
    /// * `principal_arn` - ARN of the principal (user, role, etc.)
    /// * `tenant_id` - Tenant ID (e.g., "acme/engineering")
    /// * `action` - The action to authorize
    ///
    /// # Returns
    ///
    /// Returns `true` if the action is allowed, `false` if denied
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::tenant::authorization::{TenantAuthorizer, TenantAction};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let policies = vec![
    ///     r#"{
    ///         "Version": "2012-10-17",
    ///         "Statement": [{
    ///             "Effect": "Allow",
    ///             "Action": "tenant:Read",
    ///             "Resource": "arn:wami:tenant::acme/*"
    ///         }]
    ///     }"#.to_string(),
    /// ];
    ///
    /// let authorizer = TenantAuthorizer::new(policies);
    /// let allowed = authorizer.check_permission(
    ///     "arn:aws:iam::123456789012:user/alice",
    ///     "acme/engineering",
    ///     TenantAction::Read,
    /// ).await?;
    ///
    /// assert!(allowed);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_permission(
        &self,
        _principal_arn: &str,
        tenant_id: &str,
        action: TenantAction,
    ) -> Result<bool> {
        let action_str = action.to_action_string();
        let resource_arn = format!("arn:wami:tenant::{}", tenant_id);

        // Evaluate policies
        let mut explicitly_denied = false;
        let mut allowed = false;

        for policy in &self.policies {
            for statement in &policy.statement {
                // Check if this statement matches our action and resource
                if self.matches_action(statement, action_str)
                    && self.matches_resource(statement, &resource_arn)
                {
                    match statement.effect.as_str() {
                        "Allow" => allowed = true,
                        "Deny" => explicitly_denied = true,
                        _ => {}
                    }
                }
            }
        }

        // Explicit deny always wins
        if explicitly_denied {
            return Ok(false);
        }

        Ok(allowed)
    }

    /// Check if an action matches a statement's actions
    fn matches_action(&self, statement: &PolicyStatement, action: &str) -> bool {
        for statement_action in &statement.action {
            if self.wildcard_match(statement_action, action) {
                return true;
            }
        }
        false
    }

    /// Check if a resource matches a statement's resources
    fn matches_resource(&self, statement: &PolicyStatement, resource: &str) -> bool {
        for statement_resource in &statement.resource {
            if self.wildcard_match(statement_resource, resource) {
                return true;
            }
        }
        false
    }

    /// Simple wildcard matching for actions and resources
    fn wildcard_match(&self, pattern: &str, value: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            return value.starts_with(prefix);
        }

        pattern == value
    }
}

/// Helper function to build a tenant admin policy
///
/// Creates a policy document that grants all permissions on a tenant and its children
///
/// # Example
///
/// ```rust
/// use wami::tenant::authorization::build_tenant_admin_policy;
///
/// let policy_json = build_tenant_admin_policy("acme");
/// println!("Admin policy: {}", policy_json);
/// ```
pub fn build_tenant_admin_policy(tenant_id: &str) -> String {
    format!(
        r#"{{
    "Version": "2012-10-17",
    "Statement": [{{
        "Effect": "Allow",
        "Action": "tenant:*",
        "Resource": "arn:wami:tenant::{}/*"
    }}]
}}"#,
        tenant_id
    )
}

/// Helper function to build a read-only tenant policy
///
/// Creates a policy document that grants only read permissions on a tenant
///
/// # Example
///
/// ```rust
/// use wami::tenant::authorization::build_tenant_readonly_policy;
///
/// let policy_json = build_tenant_readonly_policy("acme/engineering");
/// println!("Read-only policy: {}", policy_json);
/// ```
pub fn build_tenant_readonly_policy(tenant_id: &str) -> String {
    format!(
        r#"{{
    "Version": "2012-10-17",
    "Statement": [{{
        "Effect": "Allow",
        "Action": "tenant:Read",
        "Resource": "arn:wami:tenant::{}"
    }}]
}}"#,
        tenant_id
    )
}

/// Legacy compatibility function for checking tenant permissions
///
/// This function provides backward compatibility with the old authorization system.
/// For new code, prefer using `TenantAuthorizer` directly.
///
/// # Note
///
/// This function currently implements a simple hierarchical check:
/// - User must be an admin of the tenant or any parent tenant
///
/// For more sophisticated policy-based authorization, use `TenantAuthorizer`.
pub async fn check_tenant_permission<S: Store>(
    store: &mut S,
    user_arn: &str,
    tenant_id: &TenantId,
    _action: TenantAction,
) -> Result<bool> {
    // Check if user is admin of this tenant
    let tenant_store = store.tenant_store().await?;

    if let Some(tenant) = tenant_store.get_tenant(tenant_id).await? {
        if tenant.admin_principals.contains(&user_arn.to_string()) {
            return Ok(true);
        }
    }

    // Check if user is admin of any parent tenant (hierarchical permissions)
    let ancestors = tenant_store.get_ancestors(tenant_id).await?;
    for ancestor in ancestors {
        if ancestor.admin_principals.contains(&user_arn.to_string()) {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_authorizer_allow() {
        let policies = vec![r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "tenant:Read",
                "Resource": "arn:wami:tenant::acme/*"
            }]
        }"#
        .to_string()];

        let authorizer = TenantAuthorizer::new(policies);
        let allowed = authorizer
            .check_permission(
                "arn:aws:iam::123456789012:user/alice",
                "acme/engineering",
                TenantAction::Read,
            )
            .await
            .unwrap();

        assert!(allowed);
    }

    #[tokio::test]
    async fn test_tenant_authorizer_deny() {
        let policies = vec![r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "tenant:Read",
                "Resource": "arn:wami:tenant::acme/*"
            }]
        }"#
        .to_string()];

        let authorizer = TenantAuthorizer::new(policies);
        let allowed = authorizer
            .check_permission(
                "arn:aws:iam::123456789012:user/alice",
                "acme/engineering",
                TenantAction::Delete,
            )
            .await
            .unwrap();

        assert!(!allowed);
    }

    #[tokio::test]
    async fn test_tenant_authorizer_explicit_deny() {
        let policies = vec![
            r#"{
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": "Allow",
                    "Action": "tenant:*",
                    "Resource": "*"
                }]
            }"#
            .to_string(),
            r#"{
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": "Deny",
                    "Action": "tenant:Delete",
                    "Resource": "arn:wami:tenant::acme/production/*"
                }]
            }"#
            .to_string(),
        ];

        let authorizer = TenantAuthorizer::new(policies);
        let allowed = authorizer
            .check_permission(
                "arn:aws:iam::123456789012:user/alice",
                "acme/production/frontend",
                TenantAction::Delete,
            )
            .await
            .unwrap();

        assert!(!allowed);
    }

    #[tokio::test]
    async fn test_tenant_authorizer_wildcard() {
        let policies = vec![r#"{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "tenant:*",
                "Resource": "*"
            }]
        }"#
        .to_string()];

        let authorizer = TenantAuthorizer::new(policies);
        let allowed = authorizer
            .check_permission(
                "arn:aws:iam::123456789012:user/admin",
                "any/tenant/id",
                TenantAction::All,
            )
            .await
            .unwrap();

        assert!(allowed);
    }

    #[test]
    fn test_build_admin_policy() {
        let policy = build_tenant_admin_policy("acme");
        assert!(policy.contains("tenant:*"));
        assert!(policy.contains("arn:wami:tenant::acme/*"));
    }

    #[test]
    fn test_build_readonly_policy() {
        let policy = build_tenant_readonly_policy("acme/engineering");
        assert!(policy.contains("tenant:Read"));
        assert!(policy.contains("arn:wami:tenant::acme/engineering"));
    }
}
