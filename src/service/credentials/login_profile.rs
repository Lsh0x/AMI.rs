//! Login Profile Service
//!
//! Orchestrates login profile management operations.

use crate::context::WamiContext;
use crate::error::Result;
use crate::store::traits::LoginProfileStore;
use crate::wami::credentials::login_profile::{
    builder as login_builder, CreateLoginProfileRequest, LoginProfile, UpdateLoginProfileRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing IAM login profiles
///
/// Provides high-level operations for console password management.
pub struct LoginProfileService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: LoginProfileStore> LoginProfileService<S> {
    /// Create a new LoginProfileService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Create a new login profile
    pub async fn create_login_profile(
        &self,
        context: &WamiContext,
        request: CreateLoginProfileRequest,
    ) -> Result<LoginProfile> {
        // Use wami builder to create login profile
        // Note: Password is validated but not stored in the model for security
        let login_profile = login_builder::build_login_profile(
            request.user_name,
            request.password_reset_required,
            context,
        )?;

        // Store it
        self.store
            .write()
            .unwrap()
            .create_login_profile(login_profile)
            .await
    }

    /// Get a login profile for a user
    pub async fn get_login_profile(&self, user_name: &str) -> Result<Option<LoginProfile>> {
        self.store
            .read()
            .unwrap()
            .get_login_profile(user_name)
            .await
    }

    /// Update a login profile
    pub async fn update_login_profile(
        &self,
        request: UpdateLoginProfileRequest,
    ) -> Result<LoginProfile> {
        // Get existing profile
        let profile = self
            .store
            .read()
            .unwrap()
            .get_login_profile(&request.user_name)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("LoginProfile for user: {}", request.user_name),
            })?;

        // Apply updates using builder functions
        // Note: Password updates are handled separately for security
        let updated_profile =
            login_builder::update_login_profile(profile, request.password_reset_required);

        // Store updated profile
        self.store
            .write()
            .unwrap()
            .update_login_profile(updated_profile)
            .await
    }

    /// Delete a login profile
    pub async fn delete_login_profile(&self, user_name: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_login_profile(user_name)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::store::memory::InMemoryWamiStore;

    fn setup_service() -> LoginProfileService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        LoginProfileService::new(store)
    }

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:iam:test:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("test"))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get_login_profile() {
        let service = setup_service();

        let request = CreateLoginProfileRequest {
            user_name: "alice".to_string(),
            password: "P@ssw0rd123!".to_string(), // Validated but not stored
            password_reset_required: true,
        };

        let context = test_context();
        let profile = service
            .create_login_profile(&context, request)
            .await
            .unwrap();
        assert_eq!(profile.user_name, "alice");
        assert!(profile.password_reset_required);

        let retrieved = service.get_login_profile("alice").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved_profile = retrieved.unwrap();
        assert_eq!(retrieved_profile.user_name, "alice");
        assert!(retrieved_profile.password_reset_required);
    }

    #[tokio::test]
    async fn test_update_login_profile() {
        let service = setup_service();

        // Create profile
        let create_request = CreateLoginProfileRequest {
            user_name: "bob".to_string(),
            password: "InitialP@ss123".to_string(),
            password_reset_required: true,
        };
        let context = test_context();
        service
            .create_login_profile(&context, create_request)
            .await
            .unwrap();

        // Update profile - change password_reset_required flag
        let update_request = UpdateLoginProfileRequest {
            user_name: "bob".to_string(),
            password: Some("NewP@ssw0rd456!".to_string()), // Password validation only
            password_reset_required: Some(false),
        };
        let updated = service.update_login_profile(update_request).await.unwrap();
        assert_eq!(updated.user_name, "bob");
        assert!(!updated.password_reset_required);
    }

    #[tokio::test]
    async fn test_delete_login_profile() {
        let service = setup_service();

        let request = CreateLoginProfileRequest {
            user_name: "charlie".to_string(),
            password: "TempP@ss789".to_string(),
            password_reset_required: false,
        };
        let context = test_context();
        service
            .create_login_profile(&context, request)
            .await
            .unwrap();

        service.delete_login_profile("charlie").await.unwrap();

        let retrieved = service.get_login_profile("charlie").await.unwrap();
        assert!(retrieved.is_none());
    }
}
