//! AccessKey Builder

use super::model::AccessKey;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::provider::ProviderConfig;
use uuid::Uuid;

/// Build a new AccessKey resource with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_access_key(user_name: String, context: &WamiContext) -> Result<AccessKey> {
    // Generate AWS-compatible access key ID (AKIA prefix + 16 alphanumeric chars)
    let random_part = Uuid::new_v4().to_string().replace('-', "").to_uppercase();
    let access_key_id = format!("AKIA{}", &random_part[..16]);

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("access-key", &access_key_id)
        .build()?;

    // Generate secret access key (40 random chars)
    let secret_access_key = uuid::Uuid::new_v4().to_string().replace('-', "")
        + &uuid::Uuid::new_v4().to_string().replace('-', "")[..8];

    Ok(AccessKey {
        user_name,
        access_key_id,
        status: "Active".to_string(),
        create_date: chrono::Utc::now(),
        secret_access_key: Some(secret_access_key),
        wami_arn,
        providers: Vec::new(),
    })
}

/// Update an AccessKey's status
pub fn update_access_key_status(mut access_key: AccessKey, new_status: String) -> AccessKey {
    access_key.status = new_status;
    access_key
}

/// Add a provider configuration to an AccessKey
pub fn add_provider_to_access_key(mut access_key: AccessKey, config: ProviderConfig) -> AccessKey {
    access_key.providers.push(config);
    access_key
}
