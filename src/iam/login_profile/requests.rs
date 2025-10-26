//! LoginProfile Request and Response Types

use serde::{Deserialize, Serialize};

/// Request to create a login profile (console password) for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoginProfileRequest {
    /// The name of the user to create a login profile for
    pub user_name: String,
    /// The new password for the user
    pub password: String,
    /// Whether the user must reset their password on next sign-in
    #[serde(default)]
    pub password_reset_required: bool,
}

/// Request to update a login profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLoginProfileRequest {
    /// The name of the user whose login profile to update
    pub user_name: String,
    /// The new password (optional)
    pub password: Option<String>,
    /// Whether the user must reset their password on next sign-in (optional)
    pub password_reset_required: Option<bool>,
}

/// Request to get a login profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLoginProfileRequest {
    /// The name of the user whose login profile to get
    pub user_name: String,
}
