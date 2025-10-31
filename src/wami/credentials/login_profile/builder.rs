//! LoginProfile Builder

use super::model::LoginProfile;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::provider::ProviderConfig;

/// Build a new LoginProfile resource with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_login_profile(
    user_name: String,
    password_reset_required: bool,
    context: &WamiContext,
) -> Result<LoginProfile> {
    // Build WAMI ARN using context (login profile uses user ARN pattern)
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("user", &user_name)
        .build()?;

    Ok(LoginProfile {
        user_name,
        create_date: chrono::Utc::now(),
        password_reset_required,
        wami_arn,
        providers: Vec::new(),
    })
}

/// Update a LoginProfile resource
pub fn update_login_profile(
    mut login_profile: LoginProfile,
    password_reset_required: Option<bool>,
) -> LoginProfile {
    if let Some(reset_required) = password_reset_required {
        login_profile.password_reset_required = reset_required;
    }
    login_profile
}

/// Add a provider configuration to a LoginProfile
pub fn add_provider_to_login_profile(
    mut login_profile: LoginProfile,
    config: ProviderConfig,
) -> LoginProfile {
    login_profile.providers.push(config);
    login_profile
}
