//! AccessKey Builder

use super::model::AccessKey;
use crate::provider::{CloudProvider, ProviderConfig, ResourceType};

/// Build a new AccessKey resource
pub fn build_access_key(
    user_name: String,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> AccessKey {
    let access_key_id = provider.generate_resource_id(ResourceType::AccessKey);
    let wami_arn =
        provider.generate_wami_arn(ResourceType::AccessKey, account_id, "/", &access_key_id);

    // Generate secret access key (40 random chars)
    let secret_access_key = uuid::Uuid::new_v4().to_string().replace('-', "")
        + &uuid::Uuid::new_v4().to_string().replace('-', "")[..8];

    AccessKey {
        user_name,
        access_key_id,
        status: "Active".to_string(),
        create_date: chrono::Utc::now(),
        secret_access_key: Some(secret_access_key),
        wami_arn,
        providers: Vec::new(),
    }
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
