//! LoginProfile Builder

use super::model::LoginProfile;
use crate::provider::{CloudProvider, ProviderConfig, ResourceType};

/// Build a new LoginProfile resource
pub fn build_login_profile(
    user_name: String,
    password_reset_required: bool,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> LoginProfile {
    let wami_arn = provider.generate_wami_arn(ResourceType::User, account_id, "/", &user_name);

    LoginProfile {
        user_name,
        create_date: chrono::Utc::now(),
        password_reset_required,
        wami_arn,
        providers: Vec::new(),
    }
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
