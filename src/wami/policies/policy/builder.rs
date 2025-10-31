//! Policy Builder

use super::model::Policy;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::provider::ProviderConfig;
use crate::types::Tag;
use uuid::Uuid;

/// Build a new Policy resource with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_policy(
    policy_name: String,
    policy_document: String,
    path: Option<String>,
    description: Option<String>,
    tags: Option<Vec<Tag>>,
    context: &WamiContext,
) -> Result<Policy> {
    let path = path.unwrap_or_else(|| "/".to_string());
    let policy_id = Uuid::new_v4().to_string();

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("policy", &policy_id)
        .build()?;

    // Generate AWS-compatible ARN (for backward compatibility)
    let arn = format!(
        "arn:aws:iam::{}:policy{}/{}",
        context.instance_id(),
        if path == "/" { "" } else { &path },
        policy_name
    );

    Ok(Policy {
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
    })
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
