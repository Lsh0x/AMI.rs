//! Tag Domain Operations - Pure Functions

use crate::error::{AmiError, Result};
use crate::types::Tag;

/// Pure domain operations for tags
pub mod tag_operations {
    use super::*;

    /// Validate a single tag (pure function)
    #[allow(clippy::result_large_err)]
    pub fn validate_tag(tag: &Tag) -> Result<()> {
        validate_tag_key(&tag.key)?;
        validate_tag_value(&tag.value)?;
        Ok(())
    }

    /// Validate tag key format (pure function)
    #[allow(clippy::result_large_err)]
    pub fn validate_tag_key(key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Tag key cannot be empty".to_string(),
            });
        }

        if key.len() > 128 {
            return Err(AmiError::InvalidParameter {
                message: "Tag key cannot exceed 128 characters".to_string(),
            });
        }

        // AWS tag key restrictions
        if key.starts_with("aws:") {
            return Err(AmiError::InvalidParameter {
                message: "Tag keys cannot start with 'aws:' (reserved prefix)".to_string(),
            });
        }

        Ok(())
    }

    /// Validate tag value format (pure function)
    #[allow(clippy::result_large_err)]
    pub fn validate_tag_value(value: &str) -> Result<()> {
        if value.len() > 256 {
            return Err(AmiError::InvalidParameter {
                message: "Tag value cannot exceed 256 characters".to_string(),
            });
        }

        Ok(())
    }

    /// Validate a list of tags (pure function)
    #[allow(clippy::result_large_err)]
    pub fn validate_tags(tags: &[Tag]) -> Result<()> {
        if tags.len() > 50 {
            return Err(AmiError::InvalidParameter {
                message: "Cannot have more than 50 tags per resource".to_string(),
            });
        }

        // Check for duplicate keys
        let mut keys = std::collections::HashSet::new();
        for tag in tags {
            if !keys.insert(&tag.key) {
                return Err(AmiError::InvalidParameter {
                    message: format!("Duplicate tag key: {}", tag.key),
                });
            }
            validate_tag(tag)?;
        }

        Ok(())
    }

    /// Merge tags, with new tags overwriting existing ones (pure function)
    pub fn merge_tags(existing: Vec<Tag>, new_tags: Vec<Tag>) -> Vec<Tag> {
        let mut tag_map: std::collections::HashMap<String, String> =
            existing.into_iter().map(|t| (t.key, t.value)).collect();

        for tag in new_tags {
            tag_map.insert(tag.key, tag.value);
        }

        tag_map
            .into_iter()
            .map(|(key, value)| Tag { key, value })
            .collect()
    }

    /// Remove tags by key (pure function)
    pub fn remove_tags(tags: Vec<Tag>, keys_to_remove: &[String]) -> Vec<Tag> {
        tags.into_iter()
            .filter(|tag| !keys_to_remove.contains(&tag.key))
            .collect()
    }

    /// Find tag by key (pure function)
    pub fn find_tag<'a>(tags: &'a [Tag], key: &str) -> Option<&'a Tag> {
        tags.iter().find(|tag| tag.key == key)
    }

    /// Check if tags contain a specific key (pure function)
    pub fn has_tag(tags: &[Tag], key: &str) -> bool {
        tags.iter().any(|tag| tag.key == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tag_operations::*;

    fn make_tag(key: &str, value: &str) -> Tag {
        Tag {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn test_validate_tag_valid() {
        let tag = make_tag("Environment", "Production");
        assert!(validate_tag(&tag).is_ok());
    }

    #[test]
    fn test_validate_tag_key_empty() {
        let result = validate_tag_key("");
        assert!(result.is_err());
        assert!(matches!(result, Err(AmiError::InvalidParameter { .. })));
    }

    #[test]
    fn test_validate_tag_key_too_long() {
        let long_key = "k".repeat(129);
        let result = validate_tag_key(&long_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tag_key_aws_prefix() {
        let result = validate_tag_key("aws:something");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tag_key_valid() {
        assert!(validate_tag_key("Environment").is_ok());
        assert!(validate_tag_key("project-name").is_ok());
        assert!(validate_tag_key("team_123").is_ok());
        assert!(validate_tag_key("a").is_ok());
    }

    #[test]
    fn test_validate_tag_value_valid() {
        assert!(validate_tag_value("").is_ok());
        assert!(validate_tag_value("value").is_ok());
        assert!(validate_tag_value("a".repeat(256).as_str()).is_ok());
    }

    #[test]
    fn test_validate_tag_value_too_long() {
        let long_value = "v".repeat(257);
        let result = validate_tag_value(&long_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tags_empty() {
        assert!(validate_tags(&[]).is_ok());
    }

    #[test]
    fn test_validate_tags_valid() {
        let tags = vec![
            make_tag("Env", "Prod"),
            make_tag("Team", "Backend"),
            make_tag("Project", "WAMI"),
        ];
        assert!(validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_validate_tags_too_many() {
        let tags: Vec<Tag> = (0..51)
            .map(|i| make_tag(&format!("key{}", i), "value"))
            .collect();
        let result = validate_tags(&tags);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tags_duplicate_keys() {
        let tags = vec![make_tag("Env", "Prod"), make_tag("Env", "Dev")];
        let result = validate_tags(&tags);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_tags_no_overlap() {
        let existing = vec![make_tag("A", "1"), make_tag("B", "2")];
        let new_tags = vec![make_tag("C", "3")];

        let merged = merge_tags(existing, new_tags);

        assert_eq!(merged.len(), 3);
        assert!(has_tag(&merged, "A"));
        assert!(has_tag(&merged, "B"));
        assert!(has_tag(&merged, "C"));
    }

    #[test]
    fn test_merge_tags_with_overlap() {
        let existing = vec![make_tag("A", "1"), make_tag("B", "2")];
        let new_tags = vec![make_tag("B", "updated"), make_tag("C", "3")];

        let merged = merge_tags(existing, new_tags);

        assert_eq!(merged.len(), 3);
        let tag_b = find_tag(&merged, "B").unwrap();
        assert_eq!(tag_b.value, "updated");
    }

    #[test]
    fn test_merge_tags_empty() {
        let existing = vec![make_tag("A", "1")];
        let merged = merge_tags(existing.clone(), vec![]);
        assert_eq!(merged.len(), 1);

        let merged2 = merge_tags(vec![], existing);
        assert_eq!(merged2.len(), 1);
    }

    #[test]
    fn test_remove_tags() {
        let tags = vec![make_tag("A", "1"), make_tag("B", "2"), make_tag("C", "3")];

        let keys_to_remove = vec!["B".to_string()];
        let remaining = remove_tags(tags, &keys_to_remove);

        assert_eq!(remaining.len(), 2);
        assert!(has_tag(&remaining, "A"));
        assert!(!has_tag(&remaining, "B"));
        assert!(has_tag(&remaining, "C"));
    }

    #[test]
    fn test_remove_tags_multiple() {
        let tags = vec![make_tag("A", "1"), make_tag("B", "2"), make_tag("C", "3")];

        let keys_to_remove = vec!["A".to_string(), "C".to_string()];
        let remaining = remove_tags(tags, &keys_to_remove);

        assert_eq!(remaining.len(), 1);
        assert!(has_tag(&remaining, "B"));
    }

    #[test]
    fn test_remove_tags_none() {
        let tags = vec![make_tag("A", "1"), make_tag("B", "2")];
        let remaining = remove_tags(tags, &[]);
        assert_eq!(remaining.len(), 2);
    }

    #[test]
    fn test_find_tag_exists() {
        let tags = vec![make_tag("A", "1"), make_tag("B", "2")];
        let tag = find_tag(&tags, "B");
        assert!(tag.is_some());
        assert_eq!(tag.unwrap().value, "2");
    }

    #[test]
    fn test_find_tag_not_exists() {
        let tags = vec![make_tag("A", "1")];
        let tag = find_tag(&tags, "Z");
        assert!(tag.is_none());
    }

    #[test]
    fn test_has_tag() {
        let tags = vec![make_tag("Env", "Prod"), make_tag("Team", "Backend")];

        assert!(has_tag(&tags, "Env"));
        assert!(has_tag(&tags, "Team"));
        assert!(!has_tag(&tags, "Project"));
    }

    #[test]
    fn test_has_tag_empty() {
        let tags: Vec<Tag> = vec![];
        assert!(!has_tag(&tags, "anything"));
    }
}
