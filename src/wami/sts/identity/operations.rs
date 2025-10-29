//! STS Identity Operations - Pure Functions

use super::model::CallerIdentity;

/// Pure domain operations for STS identity
pub mod identity_operations {
    use super::*;

    /// Build caller identity from ARN (pure function)
    pub fn build_identity(arn: String, account_id: String, user_id: String) -> CallerIdentity {
        CallerIdentity {
            arn: arn.clone(),
            account: account_id,
            user_id,
            wami_arn: arn, // Use same ARN for now
            providers: vec![],
        }
    }

    /// Extract account ID from ARN (pure function)
    pub fn extract_account_from_arn(arn: &str) -> Option<String> {
        // ARN format: arn:partition:service:region:account-id:resource
        let parts: Vec<&str> = arn.split(':').collect();
        if parts.len() >= 5 {
            Some(parts[4].to_string())
        } else {
            None
        }
    }

    /// Extract resource name from ARN (pure function)
    pub fn extract_name_from_arn(arn: &str) -> Option<String> {
        // ARN format: arn:partition:service:region:account-id:resource-type/resource-name
        let parts: Vec<&str> = arn.split('/').collect();
        if parts.len() >= 2 {
            Some(parts[parts.len() - 1].to_string())
        } else {
            None
        }
    }

    /// Validate ARN format (pure function)
    pub fn is_valid_arn(arn: &str) -> bool {
        let parts: Vec<&str> = arn.split(':').collect();
        parts.len() >= 6 && parts[0] == "arn"
    }

    /// Extract service from ARN (pure function)
    pub fn extract_service_from_arn(arn: &str) -> Option<String> {
        let parts: Vec<&str> = arn.split(':').collect();
        if parts.len() >= 3 {
            Some(parts[2].to_string())
        } else {
            None
        }
    }

    /// Extract region from ARN (pure function)
    pub fn extract_region_from_arn(arn: &str) -> Option<String> {
        let parts: Vec<&str> = arn.split(':').collect();
        if parts.len() >= 4 {
            let region = parts[3].to_string();
            if region.is_empty() {
                None
            } else {
                Some(region)
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use identity_operations::*;

    #[test]
    fn test_build_identity() {
        let identity = build_identity(
            "arn:aws:iam::123456789012:user/alice".to_string(),
            "123456789012".to_string(),
            "AIDAI123456".to_string(),
        );

        assert_eq!(identity.arn, "arn:aws:iam::123456789012:user/alice");
        assert_eq!(identity.account, "123456789012");
        assert_eq!(identity.user_id, "AIDAI123456");
    }

    #[test]
    fn test_extract_account_from_arn() {
        let arn = "arn:aws:iam::123456789012:user/alice";
        let account = extract_account_from_arn(arn);
        assert_eq!(account, Some("123456789012".to_string()));
    }

    #[test]
    fn test_extract_account_from_arn_invalid() {
        let arn = "invalid:arn";
        let account = extract_account_from_arn(arn);
        assert_eq!(account, None);
    }

    #[test]
    fn test_extract_name_from_arn() {
        let arn = "arn:aws:iam::123456789012:user/alice";
        let name = extract_name_from_arn(arn);
        assert_eq!(name, Some("alice".to_string()));
    }

    #[test]
    fn test_extract_name_from_arn_no_slash() {
        let arn = "arn:aws:iam::123456789012:user";
        let name = extract_name_from_arn(arn);
        assert_eq!(name, None);
    }

    #[test]
    fn test_is_valid_arn() {
        assert!(is_valid_arn("arn:aws:iam::123456789012:user/alice"));
        assert!(is_valid_arn("arn:wami:identity:hash:tenant:user/bob"));
        assert!(is_valid_arn(
            "arn:aws:s3:us-west-2:123456789012:bucket/my-bucket"
        ));
        assert!(!is_valid_arn("not:an:arn"));
        assert!(!is_valid_arn("arn:aws:iam"));
        assert!(!is_valid_arn(""));
    }

    #[test]
    fn test_extract_service_from_arn() {
        let arn = "arn:aws:iam::123456789012:user/alice";
        let service = extract_service_from_arn(arn);
        assert_eq!(service, Some("iam".to_string()));
    }

    #[test]
    fn test_extract_service_from_arn_invalid() {
        let arn = "invalid";
        let service = extract_service_from_arn(arn);
        assert_eq!(service, None);
    }

    #[test]
    fn test_extract_region_from_arn_with_region() {
        let arn = "arn:aws:s3:us-west-2:123456789012:bucket/my-bucket";
        let region = extract_region_from_arn(arn);
        assert_eq!(region, Some("us-west-2".to_string()));
    }

    #[test]
    fn test_extract_region_from_arn_global() {
        let arn = "arn:aws:iam::123456789012:user/alice";
        let region = extract_region_from_arn(arn);
        assert_eq!(region, None);
    }

    #[test]
    fn test_extract_region_from_arn_invalid() {
        let arn = "invalid";
        let region = extract_region_from_arn(arn);
        assert_eq!(region, None);
    }
}
