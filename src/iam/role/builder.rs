//! Role Builder

use super::model::Role;
use crate::provider::{CloudProvider, ProviderConfig, ResourceType};
use crate::types::Tag;

/// Build a new Role resource
#[allow(clippy::too_many_arguments)]
pub fn build_role(
    role_name: String,
    assume_role_policy_document: String,
    path: Option<String>,
    description: Option<String>,
    max_session_duration: Option<i32>,
    permissions_boundary: Option<String>,
    tags: Option<Vec<Tag>>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Role {
    let path = path.unwrap_or_else(|| "/".to_string());
    let role_id = provider.generate_resource_id(ResourceType::Role);
    let arn =
        provider.generate_resource_identifier(ResourceType::Role, account_id, &path, &role_name);
    let wami_arn = provider.generate_wami_arn(ResourceType::Role, account_id, &path, &role_name);

    Role {
        role_name,
        role_id,
        arn,
        path,
        create_date: chrono::Utc::now(),
        assume_role_policy_document,
        description,
        max_session_duration,
        permissions_boundary,
        tags: tags.unwrap_or_default(),
        wami_arn,
        providers: Vec::new(),
        tenant_id: None,
    }
}

/// Update a Role resource with new values
pub fn update_role(
    mut role: Role,
    description: Option<String>,
    max_session_duration: Option<i32>,
) -> Role {
    if let Some(desc) = description {
        role.description = Some(desc);
    }
    if let Some(duration) = max_session_duration {
        role.max_session_duration = Some(duration);
    }
    role
}

/// Add a provider configuration to a Role
pub fn add_provider_to_role(mut role: Role, config: ProviderConfig) -> Role {
    role.providers.push(config);
    role
}
