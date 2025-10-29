//! Policy Builder

use super::model::Policy;
use crate::provider::arn_builder::WamiArnBuilder;
use crate::provider::{CloudProvider, ProviderConfig, ResourceType};
use crate::types::Tag;

/// Build a new Policy resource
pub fn build_policy(
    policy_name: String,
    policy_document: String,
    path: Option<String>,
    description: Option<String>,
    tags: Option<Vec<Tag>>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Policy {
    let path = path.unwrap_or_else(|| "/".to_string());
    let policy_id = provider.generate_resource_id(ResourceType::Policy);
    let arn = provider.generate_resource_identifier(
        ResourceType::Policy,
        account_id,
        &path,
        &policy_name,
    );

    // Generate WAMI ARN with opaque tenant hash
    let arn_builder = WamiArnBuilder::new();
    let wami_arn = arn_builder.build_arn("iam", account_id, "policy", &path, &policy_name);

    Policy {
        policy_name,
        policy_id,
        arn,
        path,
        default_version_id: "v1".to_string(),
        policy_document,
        attachment_count: 0,
        permissions_boundary_usage_count: 0,
        is_attachable: true,
        description,
        create_date: chrono::Utc::now(),
        update_date: chrono::Utc::now(),
        tags: tags.unwrap_or_default(),
        wami_arn,
        providers: Vec::new(),
        tenant_id: None,
    }
}

/// Update a Policy resource with new values
pub fn update_policy(
    mut policy: Policy,
    description: Option<String>,
    default_version_id: Option<String>,
) -> Policy {
    if let Some(desc) = description {
        policy.description = Some(desc);
    }
    if let Some(version_id) = default_version_id {
        policy.default_version_id = version_id;
    }
    policy.update_date = chrono::Utc::now();
    policy
}

/// Add a provider configuration to a Policy
pub fn add_provider_to_policy(mut policy: Policy, config: ProviderConfig) -> Policy {
    policy.providers.push(config);
    policy
}
