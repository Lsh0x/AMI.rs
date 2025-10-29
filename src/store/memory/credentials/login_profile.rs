//! Login Profile Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::LoginProfileStore;
use crate::wami::credentials::LoginProfile;
use async_trait::async_trait;

#[async_trait]
impl LoginProfileStore for InMemoryWamiStore {
    async fn create_login_profile(&mut self, profile: LoginProfile) -> Result<LoginProfile> {
        self.login_profiles
            .insert(profile.user_name.clone(), profile.clone());
        Ok(profile)
    }

    async fn get_login_profile(&self, user_name: &str) -> Result<Option<LoginProfile>> {
        Ok(self.login_profiles.get(user_name).cloned())
    }

    async fn update_login_profile(&mut self, profile: LoginProfile) -> Result<LoginProfile> {
        self.login_profiles
            .insert(profile.user_name.clone(), profile.clone());
        Ok(profile)
    }

    async fn delete_login_profile(&mut self, user_name: &str) -> Result<()> {
        self.login_profiles.remove(user_name);
        Ok(())
    }
}
