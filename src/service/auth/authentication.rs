//! Authentication Service - Credential validation and context creation
//!
//! This service handles the authentication flow:
//! 1. Validate access key credentials
//! 2. Load the user who owns the access key
//! 3. Extract instance_id and tenant_path from user's ARN
//! 4. Create WamiContext for authorized operations
//!
//! # Example
//!
//! ```rust,no_run
//! use wami::{AuthenticationService, store::memory::InMemoryWamiStore};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
//!     let auth_service = AuthenticationService::new(store);
//!
//!     // Authenticate with access key credentials
//!     let context = auth_service
//!         .authenticate("AKIAIOSFODNN7EXAMPLE", "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
//!         .await?;
//!
//!     println!("Authenticated as: {}", context.caller_arn());
//!     println!("Instance: {}", context.instance_id());
//!     println!("Tenant: {}", context.tenant_path());
//!
//!     Ok(())
//! }
//! ```

use crate::arn::TenantPath;
use crate::context::WamiContext;
use crate::error::{AmiError, Result};
use crate::store::traits::{AccessKeyStore, UserStore};
use crate::wami::identity::root_user::ROOT_USER_NAME;
use crate::wami::identity::User;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Authentication Service
///
/// Handles credential validation and context creation for WAMI operations.
pub struct AuthenticationService<S>
where
    S: AccessKeyStore + UserStore + Send + Sync,
{
    store: Arc<RwLock<S>>,
}

impl<S> AuthenticationService<S>
where
    S: AccessKeyStore + UserStore + Send + Sync,
{
    /// Create a new authentication service
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    /// Authenticate with access key credentials
    ///
    /// This is the main authentication method. It validates the credentials
    /// and returns a `WamiContext` that can be used for subsequent operations.
    ///
    /// # Arguments
    ///
    /// * `access_key_id` - The public access key identifier
    /// * `secret_access_key` - The secret access key
    ///
    /// # Returns
    ///
    /// A `WamiContext` containing the authenticated user's information
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The access key doesn't exist
    /// - The secret doesn't match
    /// - The access key is inactive
    /// - The user doesn't exist
    pub async fn authenticate(
        &self,
        access_key_id: &str,
        secret_access_key: &str,
    ) -> Result<WamiContext> {
        // Step 1: Validate access key and get the user
        let user = self
            .validate_access_key(access_key_id, secret_access_key)
            .await?;

        // Step 2: Create context from user
        self.create_context_from_user(&user).await
    }

    /// Validate access key credentials
    ///
    /// Checks that:
    /// - The access key exists
    /// - The secret matches (constant-time comparison)
    /// - The access key is active
    /// - The owning user exists
    ///
    /// # Returns
    ///
    /// The `User` who owns the access key
    async fn validate_access_key(
        &self,
        access_key_id: &str,
        secret_access_key: &str,
    ) -> Result<User> {
        let store = self.store.read().await;

        // Get the access key
        let access_key = store.get_access_key(access_key_id).await?.ok_or_else(|| {
            AmiError::InvalidParameter {
                message: "Invalid access key ID or secret".to_string(),
            }
        })?;

        // Check if access key is active
        if access_key.status.to_lowercase() != "active" {
            return Err(AmiError::InvalidParameter {
                message: "Access key is not active".to_string(),
            });
        }

        // Verify the secret (constant-time comparison)
        // Note: In production, secret_access_key in the model should be the hash
        // For now, we'll do a simple comparison
        let secret_matches = if let Some(stored_secret) = &access_key.secret_access_key {
            // Try bcrypt verification first (if it's hashed)
            if stored_secret.starts_with("$2") {
                // It's a bcrypt hash
                bcrypt::verify(secret_access_key, stored_secret).unwrap_or(false)
            } else {
                // Plaintext comparison (not secure, for backward compatibility)
                constant_time_compare(secret_access_key.as_bytes(), stored_secret.as_bytes())
            }
        } else {
            false
        };

        if !secret_matches {
            return Err(AmiError::InvalidParameter {
                message: "Invalid access key ID or secret".to_string(),
            });
        }

        // Get the user who owns this access key
        let user = store
            .get_user(&access_key.user_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("User {}", access_key.user_name),
            })?;

        Ok(user)
    }

    /// Create a WamiContext from an authenticated user
    ///
    /// Extracts the instance_id and tenant_path from the user's WAMI ARN
    /// and creates a context for subsequent operations.
    async fn create_context_from_user(&self, user: &User) -> Result<WamiContext> {
        let arn = &user.wami_arn;

        // Check if this is the root user
        // Root user is in the root tenant (ID = 0)
        let is_root = user.user_name == ROOT_USER_NAME
            && arn.tenant_path.root_u64() == Some(crate::wami::identity::root_user::ROOT_TENANT_ID);

        // Extract instance_id and tenant_path from the ARN
        let instance_id = arn.wami_instance_id.clone();
        let tenant_path = arn.tenant_path.clone();

        // Create the context
        WamiContext::builder()
            .instance_id(instance_id)
            .tenant_path(tenant_path)
            .caller_arn(arn.clone())
            .is_root(is_root)
            .build()
    }

    /// Create context for a root user
    ///
    /// Convenience method for authenticating as root user.
    pub async fn authenticate_root(
        &self,
        instance_id: &str,
        access_key_id: &str,
        secret_access_key: &str,
    ) -> Result<WamiContext> {
        // Validate credentials
        let user = self
            .validate_access_key(access_key_id, secret_access_key)
            .await?;

        // Verify this is actually the root user
        if user.user_name != ROOT_USER_NAME {
            return Err(AmiError::AccessDenied {
                message: "Not a root user".to_string(),
            });
        }

        // Verify instance matches
        if user.wami_arn.wami_instance_id != instance_id {
            return Err(AmiError::AccessDenied {
                message: "Instance ID mismatch".to_string(),
            });
        }

        // Create root context
        WamiContext::builder()
            .instance_id(instance_id)
            .tenant_path(TenantPath::single(
                crate::wami::identity::root_user::ROOT_TENANT_ID,
            ))
            .caller_arn(user.wami_arn.clone())
            .is_root(true)
            .build()
    }
}

/// Constant-time string comparison to prevent timing attacks
///
/// This is important for security-sensitive comparisons like secrets.
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

/// Helper function to hash a secret access key
///
/// This should be used when creating new access keys to store the hash
/// instead of the plaintext secret.
pub fn hash_secret(secret: &str) -> Result<String> {
    bcrypt::hash(secret, bcrypt::DEFAULT_COST)
        .map_err(|e| AmiError::StoreError(format!("Failed to hash secret: {}", e)))
}

/// Helper function to verify a secret against a hash
pub fn verify_secret(secret: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(secret, hash)
        .map_err(|e| AmiError::StoreError(format!("Failed to verify secret: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare(b"hello", b"hello"));
        assert!(!constant_time_compare(b"hello", b"world"));
        assert!(!constant_time_compare(b"hello", b"hello!"));
    }

    #[test]
    fn test_hash_and_verify_secret() {
        let secret = "my-super-secret-key";
        let hash = hash_secret(secret).unwrap();

        assert!(hash.starts_with("$2")); // Bcrypt hash marker
        assert!(verify_secret(secret, &hash).unwrap());
        assert!(!verify_secret("wrong-secret", &hash).unwrap());
    }
}
