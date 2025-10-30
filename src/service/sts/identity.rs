//! STS Identity Service
//!
//! Orchestrates caller identity operations.

use crate::error::{AmiError, Result};
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::{IdentityStore, UserStore};
use crate::wami::sts::CallerIdentity;
use std::sync::{Arc, RwLock};

/// Request to get caller identity (empty - identity comes from context)
#[derive(Debug, Clone, Default)]
pub struct GetCallerIdentityRequest {}

/// Response from getting caller identity
#[derive(Debug, Clone)]
pub struct GetCallerIdentityResponse {
    pub user_id: String,
    pub account: String,
    pub arn: String,
}

/// Service for managing caller identities
///
/// Provides high-level operations for identity inspection.
pub struct IdentityService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: IdentityStore + UserStore> IdentityService<S> {
    /// Create a new IdentityService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>, account_id: String) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
            account_id,
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
            account_id: self.account_id.clone(),
        }
    }

    /// Get the identity of the caller
    ///
    /// In a real implementation, this would extract the caller from the request context.
    /// For now, we accept the ARN as a parameter.
    pub async fn get_caller_identity(
        &self,
        _request: GetCallerIdentityRequest,
        caller_arn: &str,
    ) -> Result<GetCallerIdentityResponse> {
        // Try to get from identity store first
        {
            let store_guard = self.store.read().unwrap();
            if let Some(identity) = store_guard.get_identity(caller_arn).await? {
                return Ok(GetCallerIdentityResponse {
                    user_id: identity.user_id,
                    account: identity.account,
                    arn: identity.arn,
                });
            }
        } // Drop read lock

        // If not found, try to extract from user store
        // Parse ARN to get user name: arn:aws:iam::123456789012:user/alice
        let user_name = self.extract_user_name_from_arn(caller_arn)?;

        // Get user (drop read lock before creating identity)
        let user = {
            let store_guard = self.store.read().unwrap();
            store_guard.get_user(&user_name).await?
        }; // Drop read lock

        if let Some(user) = user {
            // Create and store identity
            let identity = CallerIdentity {
                user_id: user.user_id.clone(),
                account: self.account_id.clone(),
                arn: user.arn.clone(),
                wami_arn: user.wami_arn.clone(),
                providers: user.providers.clone(),
            };

            {
                let mut store_guard = self.store.write().unwrap();
                store_guard.create_identity(identity.clone()).await?;
            } // Drop write lock

            return Ok(GetCallerIdentityResponse {
                user_id: identity.user_id,
                account: identity.account,
                arn: identity.arn,
            });
        }

        Err(AmiError::ResourceNotFound {
            resource: format!("Identity for ARN: {}", caller_arn),
        })
    }

    /// List all identities
    pub async fn list_identities(&self) -> Result<Vec<CallerIdentity>> {
        self.store.read().unwrap().list_identities().await
    }

    // Helper method
    fn extract_user_name_from_arn(&self, arn: &str) -> Result<String> {
        // Expected format: arn:aws:iam::123456789012:user/alice
        let parts: Vec<&str> = arn.split(':').collect();
        if parts.len() < 6 {
            return Err(AmiError::InvalidParameter {
                message: format!("Invalid ARN format: {}", arn),
            });
        }

        let resource_part = parts[5]; // "user/alice"
        let resource_parts: Vec<&str> = resource_part.split('/').collect();

        if resource_parts.len() < 2 {
            return Err(AmiError::InvalidParameter {
                message: format!("Invalid ARN resource format: {}", arn),
            });
        }

        Ok(resource_parts[1..].join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::identity::user::builder::build_user;

    fn setup_service() -> IdentityService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        IdentityService::new(store, "123456789012".to_string())
    }

    #[tokio::test]
    async fn test_get_caller_identity_from_user() {
        let service = setup_service();
        let provider = AwsProvider::new();

        // Create a user
        let user = build_user(
            "alice".to_string(),
            Some("/".to_string()),
            &provider,
            "123456789012",
        );

        let user_arn = user.arn.clone();

        service
            .store
            .write()
            .unwrap()
            .create_user(user)
            .await
            .unwrap();

        // Get caller identity
        let request = GetCallerIdentityRequest {};
        let response = service
            .get_caller_identity(request, &user_arn)
            .await
            .unwrap();

        assert_eq!(response.account, "123456789012");
        assert!(response.arn.contains("alice"));
    }

    #[tokio::test]
    async fn test_list_identities() {
        let service = setup_service();
        let provider = AwsProvider::new();

        // Create users and get their identities
        for i in 0..3 {
            let user = build_user(
                format!("user{}", i),
                Some("/".to_string()),
                &provider,
                "123456789012",
            );
            let arn = user.arn.clone();

            service
                .store
                .write()
                .unwrap()
                .create_user(user)
                .await
                .unwrap();

            // Trigger identity creation
            let request = GetCallerIdentityRequest {};
            service.get_caller_identity(request, &arn).await.unwrap();
        }

        let identities = service.list_identities().await.unwrap();
        assert_eq!(identities.len(), 3);
    }

    #[tokio::test]
    async fn test_extract_user_name_from_arn() {
        let service = setup_service();

        let name = service
            .extract_user_name_from_arn("arn:aws:iam::123456789012:user/alice")
            .unwrap();
        assert_eq!(name, "alice");

        let name_with_path = service
            .extract_user_name_from_arn("arn:aws:iam::123456789012:user/department/team/bob")
            .unwrap();
        assert_eq!(name_with_path, "department/team/bob");
    }
}
