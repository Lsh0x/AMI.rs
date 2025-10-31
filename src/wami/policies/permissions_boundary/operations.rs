//! Permissions Boundary Operations
//!
//! Pure functions for permissions boundary validation and evaluation.

#![allow(clippy::result_large_err)]

use crate::error::{AmiError, Result};
use crate::wami::policies::Policy;

/// Validate that a policy is suitable for use as a permissions boundary
///
/// Checks that the policy exists, has an ARN (is managed), and has valid content.
pub fn validate_boundary_policy(policy: &Policy) -> Result<()> {
    // Must be a managed policy (not inline)
    if policy.arn.is_empty() {
        return Err(AmiError::InvalidParameter {
            message: "Permissions boundary must be a managed policy with an ARN".to_string(),
        });
    }

    // Policy must exist and be valid
    if policy.policy_document.is_empty() {
        return Err(AmiError::InvalidParameter {
            message: "Permissions boundary policy document cannot be empty".to_string(),
        });
    }

    // Validate policy document is valid JSON
    serde_json::from_str::<serde_json::Value>(&policy.policy_document).map_err(|e| {
        AmiError::InvalidParameter {
            message: format!("Invalid boundary policy document JSON: {}", e),
        }
    })?;

    Ok(())
}

/// Check if an action is allowed by the permissions boundary
///
/// Returns true if the action is within the boundary limits, false otherwise.
/// The permissions boundary acts as a maximum permission ceiling.
pub fn is_allowed_by_boundary(
    action: &str,
    resource: &str,
    boundary_policy: &Policy,
) -> Result<bool> {
    // Parse the policy document
    let policy_doc: serde_json::Value = serde_json::from_str(&boundary_policy.policy_document)
        .map_err(|e| AmiError::InvalidParameter {
            message: format!("Invalid boundary policy document: {}", e),
        })?;

    // Extract statements
    let statements =
        policy_doc["Statement"]
            .as_array()
            .ok_or_else(|| AmiError::InvalidParameter {
                message: "Policy document must have a Statement array".to_string(),
            })?;

    // Check if any Allow statement permits this action
    for statement in statements {
        let effect = statement["Effect"].as_str().unwrap_or("Deny");

        if effect != "Allow" {
            continue;
        }

        // Check action match
        if let Some(actions) = statement["Action"].as_str() {
            if action_matches(action, actions) {
                // Check resource match
                if let Some(resources) = statement["Resource"].as_str() {
                    if resource_matches(resource, resources) {
                        return Ok(true);
                    }
                }
            }
        } else if let Some(actions) = statement["Action"].as_array() {
            for act in actions {
                if let Some(act_str) = act.as_str() {
                    if action_matches(action, act_str) {
                        if let Some(resources) = statement["Resource"].as_str() {
                            if resource_matches(resource, resources) {
                                return Ok(true);
                            }
                        } else if let Some(resources) = statement["Resource"].as_array() {
                            for res in resources {
                                if let Some(res_str) = res.as_str() {
                                    if resource_matches(resource, res_str) {
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Not allowed by boundary
    Ok(false)
}

/// Check if action matches pattern (supports wildcards)
///
/// Patterns can be:
/// - `*` - matches all actions
/// - `service:*` - matches all actions in a service
/// - `service:Action*` - matches actions with prefix
/// - `service:Action` - exact match
pub fn action_matches(action: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return action.starts_with(prefix);
    }
    action == pattern
}

/// Check if resource matches pattern (supports wildcards)
///
/// Patterns can be:
/// - `*` - matches all resources
/// - `arn:aws:service:::resource/*` - matches resources with prefix
/// - `arn:aws:service:::resource` - exact match
pub fn resource_matches(resource: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return resource.starts_with(prefix);
    }
    resource == pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_matches_wildcard() {
        assert!(action_matches("s3:GetObject", "*"));
        assert!(action_matches("ec2:RunInstances", "*"));
    }

    #[test]
    fn test_action_matches_service_wildcard() {
        assert!(action_matches("s3:GetObject", "s3:*"));
        assert!(action_matches("s3:PutObject", "s3:*"));
        assert!(!action_matches("ec2:RunInstances", "s3:*"));
    }

    #[test]
    fn test_action_matches_prefix() {
        assert!(action_matches("s3:GetObject", "s3:Get*"));
        assert!(action_matches("s3:GetBucket", "s3:Get*"));
        assert!(!action_matches("s3:PutObject", "s3:Get*"));
    }

    #[test]
    fn test_action_matches_exact() {
        assert!(action_matches("s3:GetObject", "s3:GetObject"));
        assert!(!action_matches("s3:PutObject", "s3:GetObject"));
    }

    #[test]
    fn test_resource_matches_wildcard() {
        assert!(resource_matches("arn:aws:s3:::bucket/key", "*"));
        assert!(resource_matches(
            "arn:aws:ec2:us-east-1:123:instance/i-123",
            "*"
        ));
    }

    #[test]
    fn test_resource_matches_prefix() {
        assert!(resource_matches(
            "arn:aws:s3:::bucket/key",
            "arn:aws:s3:::bucket/*"
        ));
        assert!(resource_matches(
            "arn:aws:s3:::bucket/folder/file",
            "arn:aws:s3:::bucket/*"
        ));
        assert!(!resource_matches(
            "arn:aws:s3:::other/key",
            "arn:aws:s3:::bucket/*"
        ));
    }

    #[test]
    fn test_resource_matches_exact() {
        assert!(resource_matches(
            "arn:aws:s3:::bucket/key",
            "arn:aws:s3:::bucket/key"
        ));
        assert!(!resource_matches(
            "arn:aws:s3:::bucket/other",
            "arn:aws:s3:::bucket/key"
        ));
    }

    #[test]
    fn test_validate_boundary_policy_valid() {
        let policy = Policy {
            policy_name: "Boundary".to_string(),
            policy_id: "ANPA123".to_string(),
            arn: "arn:aws:iam::123456789012:policy/boundary".to_string(),
            path: "/".to_string(),
            default_version_id: "v1".to_string(),
            attachment_count: 0,
            permissions_boundary_usage_count: 0,
            is_attachable: true,
            description: Some("Test boundary".to_string()),
            create_date: chrono::Utc::now(),
            update_date: chrono::Utc::now(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            tags: vec![],
            wami_arn: "arn:wami:iam:root:wami:123456789012:policy/boundary"
                .parse()
                .unwrap(),
            providers: vec![],
            tenant_id: None,
        };

        assert!(validate_boundary_policy(&policy).is_ok());
    }

    #[test]
    fn test_validate_boundary_policy_no_arn() {
        let policy = Policy {
            policy_name: "Inline".to_string(),
            policy_id: "ANPA123".to_string(),
            arn: "".to_string(), // No ARN = inline policy
            path: "/".to_string(),
            default_version_id: "v1".to_string(),
            attachment_count: 0,
            permissions_boundary_usage_count: 0,
            is_attachable: true,
            description: None,
            create_date: chrono::Utc::now(),
            update_date: chrono::Utc::now(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            tags: vec![],
            wami_arn: "arn:wami:iam:root:wami:123456789012:policy/inline"
                .parse()
                .unwrap(),
            providers: vec![],
            tenant_id: None,
        };

        assert!(validate_boundary_policy(&policy).is_err());
    }

    #[test]
    fn test_validate_boundary_policy_empty_document() {
        let policy = Policy {
            policy_name: "Empty".to_string(),
            policy_id: "ANPA123".to_string(),
            arn: "arn:aws:iam::123456789012:policy/empty".to_string(),
            path: "/".to_string(),
            default_version_id: "v1".to_string(),
            attachment_count: 0,
            permissions_boundary_usage_count: 0,
            is_attachable: true,
            description: None,
            create_date: chrono::Utc::now(),
            update_date: chrono::Utc::now(),
            policy_document: "".to_string(),
            tags: vec![],
            wami_arn: "arn:wami:iam:root:wami:123456789012:policy/empty"
                .parse()
                .unwrap(),
            providers: vec![],
            tenant_id: None,
        };

        assert!(validate_boundary_policy(&policy).is_err());
    }

    #[test]
    fn test_is_allowed_by_boundary_simple_allow() {
        let policy = Policy {
            policy_name: "S3Boundary".to_string(),
            policy_id: "ANPA123".to_string(),
            arn: "arn:aws:iam::123456789012:policy/boundary".to_string(),
            path: "/".to_string(),
            default_version_id: "v1".to_string(),
            attachment_count: 0,
            permissions_boundary_usage_count: 0,
            is_attachable: true,
            description: None,
            create_date: chrono::Utc::now(),
            update_date: chrono::Utc::now(),
            policy_document: r#"{
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": "Allow",
                    "Action": "s3:*",
                    "Resource": "*"
                }]
            }"#
            .to_string(),
            tags: vec![],
            wami_arn: "arn:wami:iam:root:wami:123456789012:policy/boundary"
                .parse()
                .unwrap(),
            providers: vec![],
            tenant_id: None,
        };

        // S3 actions should be allowed
        assert!(
            is_allowed_by_boundary("s3:GetObject", "arn:aws:s3:::bucket/key", &policy).unwrap()
        );
        assert!(
            is_allowed_by_boundary("s3:PutObject", "arn:aws:s3:::bucket/key", &policy).unwrap()
        );

        // EC2 actions should not be allowed
        assert!(!is_allowed_by_boundary(
            "ec2:RunInstances",
            "arn:aws:ec2:us-east-1:123:instance/*",
            &policy
        )
        .unwrap());
    }

    #[test]
    fn test_is_allowed_by_boundary_array_actions() {
        let policy = Policy {
            policy_name: "MultiBoundary".to_string(),
            policy_id: "ANPA123".to_string(),
            arn: "arn:aws:iam::123456789012:policy/boundary".to_string(),
            path: "/".to_string(),
            default_version_id: "v1".to_string(),
            attachment_count: 0,
            permissions_boundary_usage_count: 0,
            is_attachable: true,
            description: None,
            create_date: chrono::Utc::now(),
            update_date: chrono::Utc::now(),
            policy_document: r#"{
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": "Allow",
                    "Action": ["s3:GetObject", "s3:PutObject"],
                    "Resource": ["arn:aws:s3:::bucket/*"]
                }]
            }"#
            .to_string(),
            tags: vec![],
            wami_arn: "arn:wami:iam:root:wami:123456789012:policy/boundary"
                .parse()
                .unwrap(),
            providers: vec![],
            tenant_id: None,
        };

        assert!(
            is_allowed_by_boundary("s3:GetObject", "arn:aws:s3:::bucket/key", &policy).unwrap()
        );
        assert!(
            is_allowed_by_boundary("s3:PutObject", "arn:aws:s3:::bucket/key", &policy).unwrap()
        );
        assert!(
            !is_allowed_by_boundary("s3:DeleteObject", "arn:aws:s3:::bucket/key", &policy).unwrap()
        );
    }
}
