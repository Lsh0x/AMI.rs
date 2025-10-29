//! Policy Domain Operations
//!
//! Pure business logic functions for policy management.

use super::{builder, model::Policy, requests::*};
use crate::error::{AmiError, Result};
use crate::provider::CloudProvider;
use crate::types::PolicyDocument;
use crate::wami::tenant::TenantId;

/// Pure domain operations for policies
pub mod policy_operations {
    use super::*;

    /// Build a new policy from a request (pure function)
    pub fn build_from_request(
        request: CreatePolicyRequest,
        provider: &dyn CloudProvider,
        account_id: &str,
        tenant_id: Option<TenantId>,
    ) -> Policy {
        builder::build_policy(
            request.policy_name,
            request.policy_document,
            request.path,
            request.description,
            request.tags,
            provider,
            account_id,
            tenant_id,
        )
    }

    /// Validate policy document format (pure function)
    pub fn validate_policy_document(doc: &PolicyDocument) -> Result<()> {
        if doc.statement.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Policy document must contain at least one statement".to_string(),
            });
        }

        for (i, statement) in doc.statement.iter().enumerate() {
            if statement.effect != "Allow" && statement.effect != "Deny" {
                return Err(AmiError::InvalidParameter {
                    message: format!(
                        "Statement {} has invalid effect '{}'. Must be 'Allow' or 'Deny'",
                        i, statement.effect
                    ),
                });
            }

            if statement.action.is_empty() {
                return Err(AmiError::InvalidParameter {
                    message: format!("Statement {} has no actions specified", i),
                });
            }

            if statement.resource.is_empty() {
                return Err(AmiError::InvalidParameter {
                    message: format!("Statement {} has no resources specified", i),
                });
            }
        }

        Ok(())
    }

    /// Check if policy belongs to tenant (pure predicate)
    pub fn belongs_to_tenant(policy: &Policy, tenant_id: &TenantId) -> bool {
        policy.tenant_id.as_ref() == Some(tenant_id)
    }

    /// Filter policies by tenant (pure function)
    pub fn filter_by_tenant(policies: Vec<Policy>, tenant_id: &TenantId) -> Vec<Policy> {
        policies
            .into_iter()
            .filter(|p| belongs_to_tenant(p, tenant_id))
            .collect()
    }

    /// Validate policy exists and belongs to tenant
    pub fn validate_policy_access(
        policy: Option<Policy>,
        policy_arn: &str,
        tenant_id: &TenantId,
    ) -> Result<Policy> {
        match policy {
            Some(p) if belongs_to_tenant(&p, tenant_id) => Ok(p),
            Some(_) => Err(AmiError::AccessDenied { message: format!("
                resource: format!("Policy: {}", policy_arn),
                reason: "Policy does not belong to current tenant".to_string(),
            }),
            None => Err(AmiError::ResourceNotFound {
                resource: format!("Policy: {}", policy_arn),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PolicyStatement;

    #[test]
    fn test_validate_policy_document() {
        let valid_doc = PolicyDocument {
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

        assert!(policy_operations::validate_policy_document(&valid_doc).is_ok());
    }

    #[test]
    fn test_validate_empty_policy() {
        let empty_doc = PolicyDocument {
            version: Some("2012-10-17".to_string()),
            statement: vec![],
        };

        assert!(policy_operations::validate_policy_document(&empty_doc).is_err());
    }

    #[test]
    fn test_validate_invalid_effect() {
        let invalid_doc = PolicyDocument {
            version: Some("2012-10-17".to_string()),
            statement: vec![PolicyStatement {
                sid: None,
                effect: "Maybe".to_string(),
                action: vec!["s3:GetObject".to_string()],
                resource: vec!["*".to_string()],
                principal: None,
                condition: None,
            }],
        };

        assert!(policy_operations::validate_policy_document(&invalid_doc).is_err());
    }
}
