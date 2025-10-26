use crate::error::Result;
use crate::iam::{IamClient, LoginProfile};
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;
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

impl<S: Store> IamClient<S> {
    /// Create a login profile (console password) for a user
    ///
    /// # Arguments
    ///
    /// * `request` - The create login profile request containing user name and password
    ///
    /// # Returns
    ///
    /// Returns the created login profile
    ///
    /// # Errors
    ///
    /// * `ResourceNotFound` - If the user doesn't exist
    /// * `ResourceExists` - If the user already has a login profile
    /// * `InvalidParameter` - If the password doesn't meet requirements
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, CreateLoginProfileRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // First create a user
    /// let user_request = CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// client.create_user(user_request).await?;
    ///
    /// // Create login profile for the user
    /// let request = CreateLoginProfileRequest {
    ///     user_name: "alice".to_string(),
    ///     password: "MySecureP@ssw0rd!".to_string(),
    ///     password_reset_required: true,
    /// };
    ///
    /// let response = client.create_login_profile(request).await?;
    /// let profile = response.data.unwrap();
    /// assert_eq!(profile.user_name, "alice");
    /// assert!(profile.password_reset_required);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_login_profile(
        &mut self,
        request: CreateLoginProfileRequest,
    ) -> Result<AmiResponse<LoginProfile>> {
        // Validate password first (before borrowing store)
        Self::validate_password(&request.password)?;

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

        // Get account ID for WAMI ARN generation
        let account_id = store.account_id();

        // Generate WAMI ARN for cross-provider identification
        let wami_arn = format!(
            "arn:wami:iam::{}:login-profile/{}",
            account_id, request.user_name
        );

        // Note: In a real implementation, you would hash the password before storing
        // For now, we'll just store it as-is (this is for mock/testing purposes)
        let profile = LoginProfile {
            user_name: request.user_name.clone(),
            create_date: chrono::Utc::now(),
            password_reset_required: request.password_reset_required,
            wami_arn,
            providers: Vec::new(),
        };

        let created_profile = store.create_login_profile(profile).await?;

        Ok(AmiResponse::success(created_profile))
    }

    /// Get a login profile for a user
    ///
    /// # Arguments
    ///
    /// * `user_name` - The name of the user whose login profile to get
    ///
    /// # Returns
    ///
    /// Returns the login profile
    ///
    /// # Errors
    ///
    /// * `ResourceNotFound` - If the user doesn't exist or doesn't have a login profile
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, CreateLoginProfileRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create user and login profile
    /// let user_request = CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// client.create_user(user_request).await?;
    ///
    /// let profile_request = CreateLoginProfileRequest {
    ///     user_name: "alice".to_string(),
    ///     password: "MySecureP@ssw0rd!".to_string(),
    ///     password_reset_required: false,
    /// };
    /// client.create_login_profile(profile_request).await?;
    ///
    /// // Get the login profile
    /// let response = client.get_login_profile("alice".to_string()).await?;
    /// let profile = response.data.unwrap();
    /// assert_eq!(profile.user_name, "alice");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `request` - The update login profile request
    ///
    /// # Returns
    ///
    /// Returns the updated login profile
    ///
    /// # Errors
    ///
    /// * `ResourceNotFound` - If the user doesn't exist or doesn't have a login profile
    /// * `InvalidParameter` - If the new password doesn't meet requirements
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, CreateLoginProfileRequest, UpdateLoginProfileRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create user and login profile
    /// let user_request = CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// client.create_user(user_request).await?;
    ///
    /// let profile_request = CreateLoginProfileRequest {
    ///     user_name: "alice".to_string(),
    ///     password: "MySecureP@ssw0rd!".to_string(),
    ///     password_reset_required: true,
    /// };
    /// client.create_login_profile(profile_request).await?;
    ///
    /// // Update the login profile
    /// let update_request = UpdateLoginProfileRequest {
    ///     user_name: "alice".to_string(),
    ///     password: Some("NewSecureP@ssw0rd!".to_string()),
    ///     password_reset_required: Some(false),
    /// };
    ///
    /// let response = client.update_login_profile(update_request).await?;
    /// let profile = response.data.unwrap();
    /// assert!(!profile.password_reset_required);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_login_profile(
        &mut self,
        request: UpdateLoginProfileRequest,
    ) -> Result<AmiResponse<LoginProfile>> {
        // Validate new password if provided (before borrowing store)
        if let Some(ref password) = request.password {
            Self::validate_password(password)?;
            // In a real implementation, you would hash the password
        }

        let store = self.iam_store().await?;

        // Get existing profile
        let mut profile = match store.get_login_profile(&request.user_name).await? {
            Some(p) => p,
            None => {
                return Err(crate::error::AmiError::ResourceNotFound {
                    resource: format!("Login profile for user: {}", request.user_name),
                });
            }
        };

        // Update password_reset_required if provided
        if let Some(reset_required) = request.password_reset_required {
            profile.password_reset_required = reset_required;
        }

        let updated_profile = store.update_login_profile(profile).await?;

        Ok(AmiResponse::success(updated_profile))
    }

    /// Delete a login profile for a user
    ///
    /// # Arguments
    ///
    /// * `user_name` - The name of the user whose login profile to delete
    ///
    /// # Returns
    ///
    /// Returns success if the login profile was deleted
    ///
    /// # Errors
    ///
    /// * `ResourceNotFound` - If the user doesn't exist or doesn't have a login profile
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, CreateLoginProfileRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create user and login profile
    /// let user_request = CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// client.create_user(user_request).await?;
    ///
    /// let profile_request = CreateLoginProfileRequest {
    ///     user_name: "alice".to_string(),
    ///     password: "MySecureP@ssw0rd!".to_string(),
    ///     password_reset_required: false,
    /// };
    /// client.create_login_profile(profile_request).await?;
    ///
    /// // Delete the login profile
    /// let response = client.delete_login_profile("alice".to_string()).await?;
    /// assert!(response.success);
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Basic requirements:
    /// - At least 8 characters
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one number
    /// - At least one special character
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
    use crate::iam::users::CreateUserRequest;
    use crate::store::memory::InMemoryStore;

    #[tokio::test]
    async fn test_create_login_profile() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user first
        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        // Create login profile
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
    async fn test_create_login_profile_already_exists() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user
        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        // Create login profile
        let request = CreateLoginProfileRequest {
            user_name: "alice".to_string(),
            password: "MySecureP@ssw0rd!".to_string(),
            password_reset_required: false,
        };
        client.create_login_profile(request.clone()).await.unwrap();

        // Try to create again
        let result = client.create_login_profile(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_login_profile_weak_password() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user
        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        // Try with weak password
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

        // Create user and login profile
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

        // Get login profile
        let response = client.get_login_profile("alice".to_string()).await.unwrap();
        let profile = response.data.unwrap();

        assert_eq!(profile.user_name, "alice");
        assert!(!profile.password_reset_required);
    }

    #[tokio::test]
    async fn test_get_login_profile_not_found() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let result = client.get_login_profile("nonexistent".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_login_profile() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user and login profile
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

        // Update login profile
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
    async fn test_update_login_profile_not_found() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let request = UpdateLoginProfileRequest {
            user_name: "nonexistent".to_string(),
            password: None,
            password_reset_required: Some(false),
        };

        let result = client.update_login_profile(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_login_profile() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user and login profile
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

        // Delete login profile
        let response = client
            .delete_login_profile("alice".to_string())
            .await
            .unwrap();
        assert!(response.success);

        // Verify it's deleted
        let result = client.get_login_profile("alice".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_login_profile_not_found() {
        let store = InMemoryStore::new();
        let mut client = IamClient::new(store);

        let result = client.delete_login_profile("nonexistent".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_password_validation() {
        // Valid password
        assert!(IamClient::<InMemoryStore>::validate_password("MySecureP@ssw0rd!").is_ok());

        // Too short
        assert!(IamClient::<InMemoryStore>::validate_password("Short1!").is_err());

        // No uppercase
        assert!(IamClient::<InMemoryStore>::validate_password("mypassword1!").is_err());

        // No lowercase
        assert!(IamClient::<InMemoryStore>::validate_password("MYPASSWORD1!").is_err());

        // No number
        assert!(IamClient::<InMemoryStore>::validate_password("MyPassword!").is_err());

        // No special character
        assert!(IamClient::<InMemoryStore>::validate_password("MyPassword1").is_err());
    }
}
