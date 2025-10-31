//! Service-Specific Credential Builder

use super::model::*;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use chrono::Utc;
use uuid::Uuid;

/// Build a new ServiceSpecificCredential resource with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_service_credential(
    user_name: String,
    service_name: String,
    context: &WamiContext,
) -> Result<ServiceSpecificCredential> {
    let credential_id = Uuid::new_v4().to_string();

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("service-credential", &credential_id)
        .build()?;

    // Generate service-specific password (for CodeCommit, IoT, etc.)
    let password = uuid::Uuid::new_v4().to_string().replace('-', "");

    let service_user_name = format!("{}-{}", user_name, &credential_id[..8]);

    Ok(ServiceSpecificCredential {
        user_name,
        service_name,
        service_user_name,
        service_password: Some(password),
        service_specific_credential_id: credential_id,
        status: "Active".to_string(),
        create_date: Utc::now(),
        wami_arn,
        providers: Vec::new(),
    })
}
