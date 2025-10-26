//! AWS Security Token Service (STS) Operations
//!
//! This module provides functionality for requesting temporary, limited-privilege credentials
//! for AWS Identity and Access Management (IAM) users or federated users.
//!
//! # Overview
//!
//! The STS module enables you to:
//!
//! - **Assume Roles**: Request temporary credentials to assume an IAM role
//! - **Get Session Tokens**: Obtain temporary credentials for IAM users with MFA
//! - **Get Federation Tokens**: Provide temporary credentials for federated users
//! - **Identity Inspection**: Get information about the calling identity
//!
//! # Example
//!
//! ```rust
//! use wami::{MemoryStsClient, sts::AssumeRoleRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = wami::create_memory_store();
//! let mut sts_client = MemoryStsClient::new(store);
//!
//! // Get caller identity
//! let identity = sts_client.get_caller_identity().await?;
//! println!("Account: {}", identity.data.unwrap().account);
//!
//! // Assume a role
//! let assume_role_request = AssumeRoleRequest {
//!     role_arn: "arn:aws:iam::123456789012:role/MyRole".to_string(),
//!     role_session_name: "my-session".to_string(),
//!     duration_seconds: Some(3600),
//!     external_id: None,
//!     policy: None,
//! };
//! let credentials = sts_client.assume_role(assume_role_request).await?;
//! println!("Temporary credentials: {:?}", credentials.data);
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::store::{Store, StsStore};

#[cfg(test)]
mod tests;

// Self-contained modules
pub mod assume_role;
pub mod credentials;
pub mod federation;
pub mod identity;
pub mod session;
pub mod session_token;

// Re-exports from sub-modules
pub use assume_role::{AssumeRoleRequest, AssumeRoleResponse, AssumedRoleUser};
pub use credentials::{CredentialType, Credentials};
pub use federation::{FederatedUser, GetFederationTokenRequest, GetFederationTokenResponse};
pub use identity::CallerIdentity;
pub use session::{SessionStatus, StsSession};
pub use session_token::GetSessionTokenRequest;

/// STS client for managing temporary AWS credentials and identity operations
///
/// The STS client provides methods for requesting temporary credentials,
/// assuming roles, and inspecting caller identity. It works with any store
/// implementation that implements the [`Store`] trait.
///
/// # Type Parameters
///
/// * `S` - The store implementation (e.g., [`InMemoryStore`](crate::store::memory::InMemoryStore))
///
/// # Example
///
/// ```rust
/// use wami::MemoryStsClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let store = wami::create_memory_store();
/// let mut sts_client = MemoryStsClient::new(store);
///
/// let identity = sts_client.get_caller_identity().await?;
/// println!("Caller identity: {:?}", identity.data);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct StsClient<S: Store> {
    pub(crate) store: S,
}

impl<S: Store> StsClient<S> {
    /// Creates a new STS client with the specified store
    ///
    /// # Arguments
    ///
    /// * `store` - The storage backend for STS resources
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{StsClient, InMemoryStore};
    ///
    /// let store = InMemoryStore::new();
    /// let sts_client = StsClient::new(store);
    /// ```
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Get mutable reference to the STS store
    pub(crate) async fn sts_store(&mut self) -> Result<&mut S::StsStore> {
        self.store.sts_store().await
    }

    /// Returns the AWS account ID associated with this client
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::MemoryStsClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut sts_client = MemoryStsClient::new(store);
    ///
    /// let account_id = sts_client.account_id().await?;
    /// println!("Account ID: {}", account_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn account_id(&mut self) -> Result<String> {
        let store = self.sts_store().await?;
        Ok(store.account_id().to_string())
    }
}
