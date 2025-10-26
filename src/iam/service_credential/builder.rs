//! Service Credential Builder

use super::model::ServiceSpecificCredential;
use crate::provider::{CloudProvider, ResourceType};
use chrono::Utc;

/// Build a new ServiceSpecificCredential resource
pub fn build_service_specific_credential(
    user_name: String,
    service_name: String,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> ServiceSpecificCredential {
    let cred_id = provider.generate_resource_id(ResourceType::ServiceCredential);

    // Generate service username (format: username-at-account_id)
    let service_user_name = format!("{}-at-{}", user_name, account_id);

    // Generate service password (random string)
    let service_password = uuid::Uuid::new_v4().to_string().replace('-', "");

    // Generate WAMI ARN for cross-provider identification
    let wami_arn =
        provider.generate_wami_arn(ResourceType::ServiceCredential, account_id, "/", &cred_id);

    ServiceSpecificCredential {
        user_name,
        service_specific_credential_id: cred_id,
        service_user_name,
        service_password: Some(service_password),
        service_name,
        create_date: Utc::now(),
        status: "Active".to_string(),
        wami_arn,
        providers: Vec::new(),
    }
}
