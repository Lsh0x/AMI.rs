//! Instance Bootstrap - Initialize WAMI instances with root user and credentials
//!
//! This module handles the secure initialization of a WAMI instance, including:
//! - Creating the root user
//! - Generating root access keys
//! - Securely hashing the secret key
//! - Returning credentials for initial authentication
//!
//! # Security Model
//!
//! **CRITICAL:** Root users MUST have access keys to authenticate. Without this,
//! anyone could brute force instance IDs and gain unauthorized root access.
//!
//! # Example
//!
//! ```rust,no_run
//! use wami::{InstanceBootstrap, store::memory::InMemoryWamiStore};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
//!     
//!     // Initialize a new WAMI instance
//!     let creds = InstanceBootstrap::initialize_instance(
//!         store.clone(),
//!         "999888777",  // instance_id
//!     ).await?;
//!     
//!     println!("Root Access Key: {}", creds.access_key_id);
//!     println!("Root Secret Key: {}", creds.secret_access_key);
//!     println!("⚠️  SAVE THESE CREDENTIALS - They cannot be retrieved later!");
//!     
//!     // Now you can authenticate as root
//!     use wami::AuthenticationService;
//!     let auth_service = AuthenticationService::new(store.clone());
//!     let context = auth_service
//!         .authenticate(&creds.access_key_id, &creds.secret_access_key)
//!         .await?;
//!     
//!     assert!(context.is_root());
//!     
//!     Ok(())
//! }
//! ```

use crate::arn::{Service, TenantPath, WamiArn};
use crate::error::{AmiError, Result};
use crate::service::auth::authentication::hash_secret;
use crate::store::traits::{AccessKeyStore, UserStore};
use crate::wami::credentials::AccessKey;
use crate::wami::identity::root_user::{ROOT_TENANT_ID, ROOT_USER_ID, ROOT_USER_NAME};
use crate::wami::identity::User;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Root user credentials returned after instance initialization
///
/// **CRITICAL SECURITY:** These credentials are shown ONCE during initialization.
/// They cannot be retrieved later. Save them securely!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCredentials {
    /// The access key ID (public identifier)
    pub access_key_id: String,

    /// The secret access key (private, like a password)
    ///
    /// **SECURITY:** This is shown in plaintext ONLY during initialization.
    /// It is stored as a bcrypt hash and cannot be retrieved later.
    pub secret_access_key: String,

    /// The instance ID this root user belongs to
    pub instance_id: String,

    /// The root user ARN
    pub user_arn: String,
}

/// Instance Bootstrap - Initialize WAMI instances
pub struct InstanceBootstrap;

impl InstanceBootstrap {
    /// Initialize a new WAMI instance with root user and credentials
    ///
    /// This creates:
    /// 1. A root user with ARN: `arn:wami:iam:0:wami:{instance_id}:user/root`
    /// 2. An access key pair for the root user
    /// 3. Securely hashed secret (bcrypt)
    ///
    /// # Security
    ///
    /// - Access key secret is hashed with bcrypt before storage
    /// - Secret is returned in plaintext ONLY during this initialization
    /// - Secrets cannot be retrieved later (by design)
    /// - Root access requires these credentials (prevents brute force attacks)
    ///
    /// # Arguments
    ///
    /// * `store` - The store to persist the root user and credentials
    /// * `instance_id` - Unique identifier for this WAMI instance
    ///
    /// # Returns
    ///
    /// `RootCredentials` containing the access key ID and secret key.
    /// **CRITICAL:** Save these credentials securely - they cannot be retrieved later!
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wami::{InstanceBootstrap, store::memory::InMemoryWamiStore};
    /// use std::sync::Arc;
    /// use tokio::sync::RwLock;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    ///     
    ///     let creds = InstanceBootstrap::initialize_instance(
    ///         store,
    ///         "999888777",
    ///     ).await?;
    ///     
    ///     // MUST save these - they're shown only once!
    ///     println!("Access Key: {}", creds.access_key_id);
    ///     println!("Secret Key: {}", creds.secret_access_key);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn initialize_instance<S>(
        store: Arc<RwLock<S>>,
        instance_id: impl Into<String>,
    ) -> Result<RootCredentials>
    where
        S: UserStore + AccessKeyStore + Send + Sync,
    {
        let instance_id = instance_id.into();

        // Validate instance_id
        if instance_id.trim().is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "instance_id cannot be empty".to_string(),
            });
        }

        let now = Utc::now();

        // Build root user ARN
        let wami_arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant_path(TenantPath::single(ROOT_TENANT_ID))
            .wami_instance(&instance_id)
            .resource("user", ROOT_USER_ID)
            .build()?;

        // Create root user
        let root_user = User {
            user_name: ROOT_USER_NAME.to_string(),
            user_id: ROOT_USER_ID.to_string(),
            wami_arn: wami_arn.clone(),
            arn: format!("arn:aws:iam::{}:user/root", instance_id),
            path: "/".to_string(),
            create_date: now,
            password_last_used: None,
            permissions_boundary: None,
            tags: vec![],
            providers: vec![],
            tenant_id: None,
        };

        // Generate access key credentials
        let access_key_id = Self::generate_access_key_id();
        let secret_access_key = Self::generate_secret_access_key();

        // Hash the secret for storage (NEVER store plaintext)
        let secret_hash = hash_secret(&secret_access_key)?;

        // Create access key ARN
        let access_key_arn = WamiArn::builder()
            .service(Service::Iam)
            .tenant_path(TenantPath::single(ROOT_TENANT_ID))
            .wami_instance(&instance_id)
            .resource("access-key", &access_key_id)
            .build()?;

        // Create access key
        let access_key = AccessKey {
            user_name: ROOT_USER_NAME.to_string(),
            access_key_id: access_key_id.clone(),
            status: "Active".to_string(),
            create_date: now,
            secret_access_key: Some(secret_hash), // Stored as hash!
            wami_arn: access_key_arn,
            providers: vec![],
        };

        // Store root user
        let mut store_guard = store.write().await;
        store_guard.create_user(root_user.clone()).await?;

        // Store access key
        store_guard.create_access_key(access_key).await?;

        // Return credentials (secret in plaintext - ONLY TIME IT'S VISIBLE!)
        Ok(RootCredentials {
            access_key_id,
            secret_access_key, // Plaintext - save this!
            instance_id,
            user_arn: wami_arn.to_string(),
        })
    }

    /// Generate a secure access key ID
    ///
    /// Format: AKIA + 16 uppercase alphanumeric characters (AWS-compatible)
    fn generate_access_key_id() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();

        let random: String = (0..16)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        format!("AKIA{}", random)
    }

    /// Generate a secure secret access key
    ///
    /// Format: 40 character alphanumeric + special chars (AWS-compatible)
    fn generate_secret_access_key() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut rng = rand::thread_rng();

        (0..40)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Check if an instance is already initialized (has a root user)
    pub async fn is_initialized<S>(store: Arc<RwLock<S>>, instance_id: &str) -> Result<bool>
    where
        S: UserStore + Send + Sync,
    {
        let store_guard = store.read().await;
        let root_user = store_guard.get_user(ROOT_USER_NAME).await?;

        // Check if root user exists and belongs to this instance
        Ok(root_user
            .map(|u| u.wami_arn.wami_instance_id == instance_id)
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::auth::AuthenticationService;
    use crate::store::memory::InMemoryWamiStore;

    #[tokio::test]
    async fn test_initialize_instance() {
        let store = Arc::new(tokio::sync::RwLock::new(InMemoryWamiStore::default()));

        let creds = InstanceBootstrap::initialize_instance(store.clone(), "999888777")
            .await
            .unwrap();

        // Verify credentials structure
        assert!(creds.access_key_id.starts_with("AKIA"));
        assert_eq!(creds.access_key_id.len(), 20);
        assert_eq!(creds.secret_access_key.len(), 40);
        assert_eq!(creds.instance_id, "999888777");

        // Verify root user was created
        let user = store
            .read()
            .await
            .get_user(ROOT_USER_NAME)
            .await
            .unwrap()
            .expect("Root user should exist");

        assert_eq!(user.user_name, ROOT_USER_NAME);
        assert_eq!(user.wami_arn.wami_instance_id, "999888777");

        // Verify access key was created
        let key = store
            .read()
            .await
            .get_access_key(&creds.access_key_id)
            .await
            .unwrap()
            .expect("Access key should exist");

        assert_eq!(key.user_name, ROOT_USER_NAME);
        assert_eq!(key.status, "Active");
    }

    #[tokio::test]
    async fn test_root_authentication() {
        let store = Arc::new(tokio::sync::RwLock::new(InMemoryWamiStore::default()));

        // Initialize instance
        let creds = InstanceBootstrap::initialize_instance(store.clone(), "999888777")
            .await
            .unwrap();

        // Authenticate as root
        let auth_service = AuthenticationService::new(store.clone());
        let context = auth_service
            .authenticate(&creds.access_key_id, &creds.secret_access_key)
            .await
            .unwrap();

        // Verify root context
        assert!(context.is_root());
        assert_eq!(context.instance_id(), "999888777");
        assert_eq!(context.tenant_path().as_string(), "0"); // Root tenant ID is 0
    }

    #[tokio::test]
    async fn test_cannot_authenticate_with_wrong_secret() {
        let store = Arc::new(tokio::sync::RwLock::new(InMemoryWamiStore::default()));

        let creds = InstanceBootstrap::initialize_instance(store.clone(), "999888777")
            .await
            .unwrap();

        // Try to authenticate with wrong secret
        let auth_service = AuthenticationService::new(store.clone());
        let result = auth_service
            .authenticate(&creds.access_key_id, "wrong_secret")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_initialized() {
        let store = Arc::new(tokio::sync::RwLock::new(InMemoryWamiStore::default()));

        // Not initialized yet
        let initialized = InstanceBootstrap::is_initialized(store.clone(), "999888777")
            .await
            .unwrap();
        assert!(!initialized);

        // Initialize
        InstanceBootstrap::initialize_instance(store.clone(), "999888777")
            .await
            .unwrap();

        // Now initialized
        let initialized = InstanceBootstrap::is_initialized(store.clone(), "999888777")
            .await
            .unwrap();
        assert!(initialized);
    }

    #[test]
    fn test_generate_access_key_id() {
        let key_id = InstanceBootstrap::generate_access_key_id();

        assert!(key_id.starts_with("AKIA"));
        assert_eq!(key_id.len(), 20);
        assert!(key_id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_secret_access_key() {
        let secret = InstanceBootstrap::generate_secret_access_key();

        assert_eq!(secret.len(), 40);
        // Should be base64-like characters
        assert!(secret
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/'));
    }
}
