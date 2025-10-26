//! User Builder
//!
//! Pure functions for building User resources without side effects.

use crate::iam::User;
use crate::provider::{CloudProvider, ProviderConfig, ResourceType};
use crate::types::Tag;

/// Build a new User resource
///
/// This is a pure function that constructs a User struct with all necessary fields.
/// It does not interact with any storage or external systems.
///
/// # Arguments
///
/// * `user_name` - The name of the user
/// * `path` - The path for the user (default: "/")
/// * `permissions_boundary` - Optional ARN of the permissions boundary policy
/// * `tags` - Optional list of tags to associate with the user
/// * `provider` - The cloud provider to use for ID and ARN generation
/// * `account_id` - The account ID for the user
///
/// # Returns
///
/// A fully constructed `User` with:
/// - Generated user_id
/// - Native ARN
/// - WAMI ARN
/// - Empty providers vector
/// - Current timestamp for create_date
pub fn build_user(
    user_name: String,
    path: Option<String>,
    permissions_boundary: Option<String>,
    tags: Option<Vec<Tag>>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> User {
    let path = path.unwrap_or_else(|| "/".to_string());
    let user_id = provider.generate_resource_id(ResourceType::User);
    let arn =
        provider.generate_resource_identifier(ResourceType::User, account_id, &path, &user_name);
    let wami_arn = provider.generate_wami_arn(ResourceType::User, account_id, &path, &user_name);

    User {
        user_name,
        user_id,
        arn,
        path,
        create_date: chrono::Utc::now(),
        password_last_used: None,
        permissions_boundary,
        tags: tags.unwrap_or_default(),
        wami_arn,
        providers: Vec::new(),
    }
}

/// Update a User resource with new values
///
/// This is a pure function that creates a new User with updated fields.
///
/// # Arguments
///
/// * `user` - The existing user to update
/// * `new_user_name` - Optional new user name
/// * `new_path` - Optional new path
/// * `provider` - The cloud provider to use for ARN regeneration
/// * `account_id` - The account ID for the user
///
/// # Returns
///
/// A new `User` with updated fields
pub fn update_user(
    mut user: User,
    new_user_name: Option<String>,
    new_path: Option<String>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> User {
    if let Some(new_name) = new_user_name {
        user.user_name = new_name.clone();
        user.arn = provider.generate_resource_identifier(
            ResourceType::User,
            account_id,
            &user.path,
            &new_name,
        );
        user.wami_arn =
            provider.generate_wami_arn(ResourceType::User, account_id, &user.path, &new_name);
    }
    if let Some(new_path) = new_path {
        user.path = new_path.clone();
        user.arn = provider.generate_resource_identifier(
            ResourceType::User,
            account_id,
            &new_path,
            &user.user_name,
        );
        user.wami_arn =
            provider.generate_wami_arn(ResourceType::User, account_id, &new_path, &user.user_name);
    }
    user
}

/// Add a provider configuration to a User
///
/// This helper function adds provider sync information to the user's providers vec.
///
/// # Arguments
///
/// * `user` - The user to update
/// * `config` - The provider configuration to add
///
/// # Returns
///
/// The updated user with the new provider configuration
pub fn add_provider_to_user(mut user: User, config: ProviderConfig) -> User {
    user.providers.push(config);
    user
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::AwsProvider;
    use std::sync::Arc;

    #[test]
    fn test_build_user() {
        let provider = Arc::new(AwsProvider::new());
        let user = build_user(
            "test-user".to_string(),
            Some("/engineering/".to_string()),
            None,
            None,
            provider.as_ref(),
            "123456789012",
        );

        assert_eq!(user.user_name, "test-user");
        assert_eq!(user.path, "/engineering/");
        assert!(user.user_id.starts_with("AIDA"));
        assert!(user
            .arn
            .contains("arn:aws:iam::123456789012:user/engineering/test-user"));
        assert!(user
            .wami_arn
            .contains("arn:wami:iam::123456789012:user/engineering/test-user"));
        assert_eq!(user.providers.len(), 0);
        assert_eq!(user.tags.len(), 0);
    }

    #[test]
    fn test_build_user_with_defaults() {
        let provider = Arc::new(AwsProvider::new());
        let user = build_user(
            "test-user".to_string(),
            None,
            None,
            None,
            provider.as_ref(),
            "123456789012",
        );

        assert_eq!(user.path, "/");
    }

    #[test]
    fn test_update_user_name() {
        let provider = Arc::new(AwsProvider::new());
        let user = build_user(
            "old-name".to_string(),
            None,
            None,
            None,
            provider.as_ref(),
            "123456789012",
        );

        let updated = update_user(
            user,
            Some("new-name".to_string()),
            None,
            provider.as_ref(),
            "123456789012",
        );

        assert_eq!(updated.user_name, "new-name");
        assert!(updated.arn.contains("new-name"));
        assert!(updated.wami_arn.contains("new-name"));
    }

    #[test]
    fn test_add_provider_to_user() {
        let provider = Arc::new(AwsProvider::new());
        let user = build_user(
            "test-user".to_string(),
            None,
            None,
            None,
            provider.as_ref(),
            "123456789012",
        );

        let config = ProviderConfig {
            provider_name: "aws".to_string(),
            account_id: "123456789012".to_string(),
            native_arn: "arn:aws:iam::123456789012:user/test-user".to_string(),
            synced_at: chrono::Utc::now(),
        };

        let updated = add_provider_to_user(user, config);
        assert_eq!(updated.providers.len(), 1);
        assert_eq!(updated.providers[0].provider_name, "aws");
    }
}
