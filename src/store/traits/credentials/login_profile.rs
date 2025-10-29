//! Login Profile Store Trait
//!
//! Focused trait for login profile storage operations

use crate::error::Result;
use crate::wami::credentials::LoginProfile;
use async_trait::async_trait;

/// Store trait for IAM login profile operations
#[async_trait]
pub trait LoginProfileStore: Send + Sync {
    /// Create a new login profile
    async fn create_login_profile(&mut self, profile: LoginProfile) -> Result<LoginProfile>;

    /// Get a login profile for a user
    async fn get_login_profile(&self, user_name: &str) -> Result<Option<LoginProfile>>;

    /// Update a login profile
    async fn update_login_profile(&mut self, profile: LoginProfile) -> Result<LoginProfile>;

    /// Delete a login profile
    async fn delete_login_profile(&mut self, user_name: &str) -> Result<()>;
}
