//! Group Builder

use super::model::Group;
use crate::provider::arn_builder::WamiArnBuilder;
use crate::provider::{CloudProvider, ProviderConfig, ResourceType};
use crate::types::Tag;

/// Build a new Group resource
pub fn build_group(
    group_name: String,
    path: Option<String>,
    tags: Option<Vec<Tag>>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Group {
    let path = path.unwrap_or_else(|| "/".to_string());
    let group_id = provider.generate_resource_id(ResourceType::Group);
    let arn =
        provider.generate_resource_identifier(ResourceType::Group, account_id, &path, &group_name);

    // Generate WAMI ARN with opaque tenant hash
    let arn_builder = WamiArnBuilder::new();
    let wami_arn = arn_builder.build_arn("iam", account_id, "group", &path, &group_name);

    Group {
        group_name,
        group_id,
        arn,
        path,
        create_date: chrono::Utc::now(),
        tags: tags.unwrap_or_default(),
        wami_arn,
        providers: Vec::new(),
        tenant_id: None,
    }
}

/// Update a Group resource with new values
pub fn update_group(
    mut group: Group,
    new_group_name: Option<String>,
    new_path: Option<String>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Group {
    let arn_builder = WamiArnBuilder::new();

    if let Some(new_name) = new_group_name {
        group.group_name = new_name.clone();
        group.arn = provider.generate_resource_identifier(
            ResourceType::Group,
            account_id,
            &group.path,
            &new_name,
        );
        group.wami_arn = arn_builder.build_arn("iam", account_id, "group", &group.path, &new_name);
    }
    if let Some(new_path) = new_path {
        group.path = new_path.clone();
        group.arn = provider.generate_resource_identifier(
            ResourceType::Group,
            account_id,
            &new_path,
            &group.group_name,
        );
        group.wami_arn =
            arn_builder.build_arn("iam", account_id, "group", &new_path, &group.group_name);
    }
    group
}

/// Add a provider configuration to a Group
pub fn add_provider_to_group(mut group: Group, config: ProviderConfig) -> Group {
    group.providers.push(config);
    group
}
