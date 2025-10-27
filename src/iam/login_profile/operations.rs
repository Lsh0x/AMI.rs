//! LoginProfile Operations

use super::{builder, model::LoginProfile, requests::*};
use crate::error::Result;
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;

impl<S: Store> IamClient<S> {
    /// Create a login profile (console password) for a user
    pub async fn create_login_profile(
        &mut self,
        request: CreateLoginProfileRequest,
    ) -> Result<AmiResponse<LoginProfile>> {
        // Validate password first (before borrowing store)
        Self::validate_password(&request.password)?;

        let account_id = self.account_id().await?;
        let provider = self.cloud_provider();

        let store = self.iam_store().await?;

        // Check if user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            });
        }

        // Check if login profile already exists
        if store.get_login_profile(&request.user_name).await?.is_some() {
            return Err(crate::error::AmiError::InvalidParameter {
                message: format!(
                    "Login profile already exists for user: {}",
                    request.user_name
                ),
            });
        }

        let profile = builder::build_login_profile(
            request.user_name,
            request.password_reset_required,
            provider.as_ref(),
            &account_id,
        );

        let created_profile = store.create_login_profile(profile).await?;

        Ok(AmiResponse::success(created_profile))
    }

    /// Get a login profile for a user
    pub async fn get_login_profile(
        &mut self,
        user_name: String,
    ) -> Result<AmiResponse<LoginProfile>> {
        let store = self.iam_store().await?;
        match store.get_login_profile(&user_name).await? {
            Some(profile) => Ok(AmiResponse::success(profile)),
            None => Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Login profile for user: {}", user_name),
            }),
        }
    }

    /// Update a login profile for a user
    pub async fn update_login_profile(
        &mut self,
        request: UpdateLoginProfileRequest,
    ) -> Result<AmiResponse<LoginProfile>> {
        // Validate new password if provided (before borrowing store)
        if let Some(ref password) = request.password {
            Self::validate_password(password)?;
        }

        let store = self.iam_store().await?;

        // Get existing profile
        let profile = match store.get_login_profile(&request.user_name).await? {
            Some(p) => p,
            None => {
                return Err(crate::error::AmiError::ResourceNotFound {
                    resource: format!("Login profile for user: {}", request.user_name),
                });
            }
        };

        let updated_profile =
            builder::update_login_profile(profile, request.password_reset_required);

        let result = store.update_login_profile(updated_profile).await?;

        Ok(AmiResponse::success(result))
    }

    /// Delete a login profile for a user
    pub async fn delete_login_profile(&mut self, user_name: String) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Check if profile exists
        if store.get_login_profile(&user_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("Login profile for user: {}", user_name),
            });
        }

        store.delete_login_profile(&user_name).await?;

        Ok(AmiResponse::success(()))
    }

    /// Validate a password against AWS IAM password policy
    #[allow(clippy::result_large_err)]
    fn validate_password(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Password must be at least 8 characters long".to_string(),
            });
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Password must contain at least one uppercase letter".to_string(),
            });
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Password must contain at least one lowercase letter".to_string(),
            });
        }

        if !password.chars().any(|c| c.is_numeric()) {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Password must contain at least one number".to_string(),
            });
        }

        if !password
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
        {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Password must contain at least one special character".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::user::CreateUserRequest;
    use crate::store::memory::InMemoryStore;

    #[tokio::test]
    async fn test_create_login_profile() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        let request = CreateLoginProfileRequest {
            user_name: "alice".to_string(),
            password: "MySecureP@ssw0rd!".to_string(),
            password_reset_required: true,
        };

        let response = client.create_login_profile(request).await.unwrap();
        let profile = response.data.unwrap();

        assert_eq!(profile.user_name, "alice");
        assert!(profile.password_reset_required);
    }

    #[tokio::test]
    async fn test_create_login_profile_user_not_found() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let request = CreateLoginProfileRequest {
            user_name: "nonexistent".to_string(),
            password: "MySecureP@ssw0rd!".to_string(),
            password_reset_required: false,
        };

        let result = client.create_login_profile(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_login_profile_weak_password() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        let request = CreateLoginProfileRequest {
            user_name: "alice".to_string(),
            password: "weak".to_string(),
            password_reset_required: false,
        };

        let result = client.create_login_profile(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_login_profile() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        let profile_request = CreateLoginProfileRequest {
            user_name: "alice".to_string(),
            password: "MySecureP@ssw0rd!".to_string(),
            password_reset_required: false,
        };
        client.create_login_profile(profile_request).await.unwrap();

        let response = client.get_login_profile("alice".to_string()).await.unwrap();
        let profile = response.data.unwrap();

        assert_eq!(profile.user_name, "alice");
        assert!(!profile.password_reset_required);
    }

    #[tokio::test]
    async fn test_update_login_profile() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        let profile_request = CreateLoginProfileRequest {
            user_name: "alice".to_string(),
            password: "MySecureP@ssw0rd!".to_string(),
            password_reset_required: true,
        };
        client.create_login_profile(profile_request).await.unwrap();

        let update_request = UpdateLoginProfileRequest {
            user_name: "alice".to_string(),
            password: Some("NewSecureP@ssw0rd!".to_string()),
            password_reset_required: Some(false),
        };

        let response = client.update_login_profile(update_request).await.unwrap();
        let profile = response.data.unwrap();

        assert_eq!(profile.user_name, "alice");
        assert!(!profile.password_reset_required);
    }

    #[tokio::test]
    async fn test_delete_login_profile() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        let profile_request = CreateLoginProfileRequest {
            user_name: "alice".to_string(),
            password: "MySecureP@ssw0rd!".to_string(),
            password_reset_required: false,
        };
        client.create_login_profile(profile_request).await.unwrap();

        let response = client
            .delete_login_profile("alice".to_string())
            .await
            .unwrap();
        assert!(response.success);

        let result = client.get_login_profile("alice".to_string()).await;
        assert!(result.is_err());
    }
}
