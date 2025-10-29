//! Service-Linked Role Domain Operations - Pure Functions

use crate::error::{AmiError, Result};

/// Pure domain operations for service-linked roles
pub mod service_linked_role_operations {
    use super::*;

    /// Validate AWS service name format (pure function)
    #[allow(clippy::result_large_err)]
    pub fn validate_service_name(service_name: &str) -> Result<()> {
        if service_name.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Service name cannot be empty".to_string(),
            });
        }

        // AWS service names typically end with .amazonaws.com
        if !service_name.ends_with(".amazonaws.com") {
            return Err(AmiError::InvalidParameter {
                message: "Service name must end with .amazonaws.com".to_string(),
            });
        }

        Ok(())
    }

    /// Validate custom suffix format (pure function)
    #[allow(clippy::result_large_err)]
    pub fn validate_custom_suffix(suffix: &str) -> Result<()> {
        if suffix.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Custom suffix cannot be empty".to_string(),
            });
        }

        if suffix.len() > 64 {
            return Err(AmiError::InvalidParameter {
                message: "Custom suffix cannot exceed 64 characters".to_string(),
            });
        }

        // Only alphanumeric and hyphens allowed
        if !suffix.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(AmiError::InvalidParameter {
                message: "Custom suffix can only contain alphanumeric characters and hyphens"
                    .to_string(),
            });
        }

        Ok(())
    }

    /// Extract service prefix from service name (pure function)
    pub fn extract_service_prefix(service_name: &str) -> String {
        service_name
            .trim_end_matches(".amazonaws.com")
            .split('.')
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// Check if service supports service-linked roles (pure function)
    pub fn is_supported_service(service_name: &str) -> bool {
        let supported_services = [
            "autoscaling.amazonaws.com",
            "elasticloadbalancing.amazonaws.com",
            "ecs.amazonaws.com",
            "rds.amazonaws.com",
            "lambda.amazonaws.com",
            "lex.amazonaws.com",
        ];

        supported_services.contains(&service_name)
    }

    /// Generate role name from service (pure function)
    pub fn generate_role_name(service_name: &str, custom_suffix: Option<&str>) -> String {
        let prefix = extract_service_prefix(service_name);
        let pascal_case = to_pascal_case(&prefix);

        if let Some(suffix) = custom_suffix {
            format!("AWSServiceRoleFor{}_{}", pascal_case, suffix)
        } else {
            format!("AWSServiceRoleFor{}", pascal_case)
        }
    }

    /// Convert kebab-case to PascalCase (pure helper)
    fn to_pascal_case(s: &str) -> String {
        s.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use service_linked_role_operations::*;

    #[test]
    fn test_validate_service_name() {
        assert!(validate_service_name("autoscaling.amazonaws.com").is_ok());
        assert!(validate_service_name("ecs.amazonaws.com").is_ok());
    }

    #[test]
    fn test_validate_service_name_empty() {
        let result = validate_service_name("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_service_name_invalid_suffix() {
        let result = validate_service_name("invalid-service");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_custom_suffix_valid() {
        assert!(validate_custom_suffix("my-suffix").is_ok());
        assert!(validate_custom_suffix("test123").is_ok());
    }

    #[test]
    fn test_validate_custom_suffix_empty() {
        let result = validate_custom_suffix("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_custom_suffix_too_long() {
        let long_suffix = "a".repeat(65);
        let result = validate_custom_suffix(&long_suffix);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_custom_suffix_invalid_chars() {
        assert!(validate_custom_suffix("invalid_suffix").is_err());
        assert!(validate_custom_suffix("invalid.suffix").is_err());
    }

    #[test]
    fn test_extract_service_prefix() {
        assert_eq!(
            extract_service_prefix("autoscaling.amazonaws.com"),
            "autoscaling"
        );
        assert_eq!(extract_service_prefix("ecs.amazonaws.com"), "ecs");
        assert_eq!(
            extract_service_prefix("elasticloadbalancing.amazonaws.com"),
            "elasticloadbalancing"
        );
    }

    #[test]
    fn test_is_supported_service() {
        assert!(is_supported_service("autoscaling.amazonaws.com"));
        assert!(is_supported_service("ecs.amazonaws.com"));
        assert!(is_supported_service("lambda.amazonaws.com"));
        assert!(!is_supported_service("unsupported.amazonaws.com"));
        assert!(!is_supported_service("random-service"));
    }

    #[test]
    fn test_generate_role_name_no_suffix() {
        let role_name = generate_role_name("autoscaling.amazonaws.com", None);
        assert_eq!(role_name, "AWSServiceRoleForAutoscaling");
    }

    #[test]
    fn test_generate_role_name_with_suffix() {
        let role_name = generate_role_name("autoscaling.amazonaws.com", Some("custom"));
        assert_eq!(role_name, "AWSServiceRoleForAutoscaling_custom");
    }

    #[test]
    fn test_generate_role_name_hyphenated_service() {
        let role_name = generate_role_name("elastic-load-balancing.amazonaws.com", None);
        assert_eq!(role_name, "AWSServiceRoleForElasticLoadBalancing");
    }
}
