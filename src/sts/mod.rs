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
//! use wami::{MemoryStsClient, AssumeRoleRequest};
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
use crate::types::AmiResponse;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// STS client for managing temporary AWS credentials and identity operations
///
/// The STS client provides methods for requesting temporary credentials,
/// assuming roles, and inspecting caller identity. It works with any store
/// implementation that implements the [`Store`] trait.
///
/// # Type Parameters
///
/// * `S` - The store implementation (e.g., [`InMemoryStore`](crate::store::in_memory::InMemoryStore))
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
    store: S,
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
    async fn sts_store(&mut self) -> Result<&mut S::StsStore> {
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

/// Represents an STS session with temporary credentials
///
/// # Example
///
/// ```rust
/// use wami::StsSession;
/// use chrono::Utc;
///
/// let session = StsSession {
///     session_token: "FwoGZXIvYXdzEBYaDH...".to_string(),
///     access_key_id: "ASIAIOSFODNN7EXAMPLE".to_string(),
///     secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
///     expiration: Utc::now() + chrono::Duration::hours(1),
///     assumed_role_arn: Some("arn:aws:iam::123456789012:role/MyRole".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StsSession {
    /// The session token for temporary credentials
    pub session_token: String,
    /// The access key ID
    pub access_key_id: String,
    /// The secret access key
    pub secret_access_key: String,
    /// When the credentials expire
    pub expiration: chrono::DateTime<chrono::Utc>,
    /// The ARN of the assumed role (if any)
    pub assumed_role_arn: Option<String>,
}

/// Information about the caller's identity
///
/// # Example
///
/// ```rust
/// use wami::CallerIdentity;
///
/// let identity = CallerIdentity {
///     user_id: "AIDACKCEVSQ6C2EXAMPLE".to_string(),
///     account: "123456789012".to_string(),
///     arn: "arn:aws:iam::123456789012:user/alice".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerIdentity {
    /// The unique identifier of the calling entity
    pub user_id: String,
    /// The AWS account ID
    pub account: String,
    /// The ARN of the calling entity
    pub arn: String,
}

/// Request to assume an IAM role
///
/// # Example
///
/// ```rust
/// use wami::AssumeRoleRequest;
///
/// let request = AssumeRoleRequest {
///     role_arn: "arn:aws:iam::123456789012:role/S3Access".to_string(),
///     role_session_name: "my-app-session".to_string(),
///     duration_seconds: Some(3600),
///     external_id: Some("unique-external-id".to_string()),
///     policy: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumeRoleRequest {
    /// The ARN of the role to assume
    pub role_arn: String,
    /// An identifier for the assumed role session
    pub role_session_name: String,
    /// The duration of the session in seconds (default: 3600, max: 43200)
    pub duration_seconds: Option<i32>,
    /// A unique identifier used by third parties for assuming a role
    pub external_id: Option<String>,
    /// An IAM policy in JSON format to further restrict permissions
    pub policy: Option<String>,
}

/// Request to get a session token
///
/// # Example
///
/// ```rust
/// use wami::GetSessionTokenRequest;
///
/// let request = GetSessionTokenRequest {
///     duration_seconds: Some(3600),
///     serial_number: Some("arn:aws:iam::123456789012:mfa/alice".to_string()),
///     token_code: Some("123456".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionTokenRequest {
    /// The duration of the session in seconds
    pub duration_seconds: Option<i32>,
    /// The identification number of the MFA device
    pub serial_number: Option<String>,
    /// The value provided by the MFA device
    pub token_code: Option<String>,
}

/// Request to get a federation token
///
/// # Example
///
/// ```rust
/// use wami::GetFederationTokenRequest;
///
/// let request = GetFederationTokenRequest {
///     name: "federated-user".to_string(),
///     duration_seconds: Some(3600),
///     policy: Some(r#"{"Version":"2012-10-17","Statement":[]}"#.to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFederationTokenRequest {
    /// The name of the federated user
    pub name: String,
    /// The duration of the session in seconds
    pub duration_seconds: Option<i32>,
    /// An IAM policy in JSON format
    pub policy: Option<String>,
}

/// Temporary AWS credentials
///
/// # Example
///
/// ```rust
/// use wami::Credentials;
/// use chrono::Utc;
///
/// let credentials = Credentials {
///     access_key_id: "ASIAIOSFODNN7EXAMPLE".to_string(),
///     secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
///     session_token: "FwoGZXIvYXdzEBYaDH...".to_string(),
///     expiration: Utc::now() + chrono::Duration::hours(1),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// The access key ID
    pub access_key_id: String,
    /// The secret access key
    pub secret_access_key: String,
    /// The session token
    pub session_token: String,
    /// When the credentials expire
    pub expiration: chrono::DateTime<chrono::Utc>,
}

impl<S: Store> StsClient<S> {
    /// Assumes an IAM role and returns temporary security credentials
    ///
    /// Returns temporary security credentials that you can use to access AWS resources.
    /// These credentials consist of an access key ID, a secret access key, and a security token.
    ///
    /// # Arguments
    ///
    /// * `request` - The assume role request parameters
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryStsClient, AssumeRoleRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut sts_client = MemoryStsClient::new(store);
    ///
    /// let request = AssumeRoleRequest {
    ///     role_arn: "arn:aws:iam::123456789012:role/DataScientist".to_string(),
    ///     role_session_name: "analytics-session".to_string(),
    ///     duration_seconds: Some(3600),
    ///     external_id: None,
    ///     policy: None,
    /// };
    ///
    /// let response = sts_client.assume_role(request).await?;
    /// let credentials = response.data.unwrap();
    /// println!("Access Key: {}", credentials.access_key_id);
    /// println!("Expires: {}", credentials.expiration);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn assume_role(
        &mut self,
        request: AssumeRoleRequest,
    ) -> Result<AmiResponse<Credentials>> {
        let session_token = format!("{}", uuid::Uuid::new_v4());
        let access_key_id = format!(
            "ASIA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );
        let secret_access_key = uuid::Uuid::new_v4().to_string().replace('-', "");

        let duration = request.duration_seconds.unwrap_or(3600);
        let expiration = chrono::Utc::now() + chrono::Duration::seconds(duration as i64);

        let credentials = Credentials {
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            session_token: session_token.clone(),
            expiration,
        };

        let session = StsSession {
            session_token: session_token.clone(),
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            expiration,
            assumed_role_arn: Some(request.role_arn),
        };

        let store = self.sts_store().await?;
        store.create_session(session).await?;

        Ok(AmiResponse::success(credentials))
    }

    /// Assume role with SAML
    pub async fn assume_role_with_saml(
        &mut self,
        role_arn: String,
        _principal_arn: String,
        _saml_assertion: String,
    ) -> Result<AmiResponse<Credentials>> {
        // In a real implementation, this would validate the SAML assertion
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name: "saml-session".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        self.assume_role(request).await
    }

    /// Assume role with web identity
    pub async fn assume_role_with_web_identity(
        &mut self,
        role_arn: String,
        _web_identity_token: String,
        role_session_name: String,
    ) -> Result<AmiResponse<Credentials>> {
        // In a real implementation, this would validate the web identity token
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name,
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        self.assume_role(request).await
    }

    /// Assume role with client grants
    pub async fn assume_role_with_client_grants(
        &mut self,
        role_arn: String,
        _client_grant_token: String,
    ) -> Result<AmiResponse<Credentials>> {
        // In a real implementation, this would validate the client grant token
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name: "client-grants-session".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        self.assume_role(request).await
    }

    /// Get federation token
    pub async fn get_federation_token(
        &mut self,
        request: GetFederationTokenRequest,
    ) -> Result<AmiResponse<Credentials>> {
        let session_token = format!("{}", uuid::Uuid::new_v4());
        let access_key_id = format!(
            "ASIA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );
        let secret_access_key = uuid::Uuid::new_v4().to_string().replace('-', "");

        let duration = request.duration_seconds.unwrap_or(3600);
        let expiration = chrono::Utc::now() + chrono::Duration::seconds(duration as i64);

        let credentials = Credentials {
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            session_token: session_token.clone(),
            expiration,
        };

        let session = StsSession {
            session_token: session_token.clone(),
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            expiration,
            assumed_role_arn: None,
        };

        let store = self.sts_store().await?;
        store.create_session(session).await?;

        Ok(AmiResponse::success(credentials))
    }

    /// Get session token
    pub async fn get_session_token(
        &mut self,
        request: Option<GetSessionTokenRequest>,
    ) -> Result<AmiResponse<Credentials>> {
        let session_token = format!("{}", uuid::Uuid::new_v4());
        let access_key_id = format!(
            "ASIA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );
        let secret_access_key = uuid::Uuid::new_v4().to_string().replace('-', "");

        let duration = request
            .as_ref()
            .and_then(|r| r.duration_seconds)
            .unwrap_or(3600);
        let expiration = chrono::Utc::now() + chrono::Duration::seconds(duration as i64);

        let credentials = Credentials {
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            session_token: session_token.clone(),
            expiration,
        };

        let session = StsSession {
            session_token: session_token.clone(),
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            expiration,
            assumed_role_arn: None,
        };

        let store = self.sts_store().await?;
        store.create_session(session).await?;

        Ok(AmiResponse::success(credentials))
    }

    /// Decode authorization message
    pub async fn decode_authorization_message(
        &self,
        encoded_message: String,
    ) -> Result<AmiResponse<String>> {
        // In a real implementation, this would decode the authorization message
        // For now, return a placeholder decoded message
        let decoded = format!("Decoded message for: {}", encoded_message);
        Ok(AmiResponse::success(decoded))
    }

    /// Get access key info
    pub async fn get_access_key_info(
        &mut self,
        _access_key_id: String,
    ) -> Result<AmiResponse<String>> {
        let store = self.sts_store().await?;
        let account_id = store.account_id();
        Ok(AmiResponse::success(account_id.to_string()))
    }

    /// Returns details about the IAM identity whose credentials are used to call this operation
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
    /// let response = sts_client.get_caller_identity().await?;
    /// let identity = response.data.unwrap();
    ///
    /// println!("User ID: {}", identity.user_id);
    /// println!("Account: {}", identity.account);
    /// println!("ARN: {}", identity.arn);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_caller_identity(&mut self) -> Result<AmiResponse<CallerIdentity>> {
        let store = self.sts_store().await?;
        let account_id = store.account_id();

        // Try to get existing identity, or create a default one
        let identity_arn = format!("arn:aws:iam::{}:user/example-user", account_id);
        let identity = store
            .get_identity(&identity_arn)
            .await?
            .unwrap_or_else(|| CallerIdentity {
                user_id: "AIDACKCEVSQ6C2EXAMPLE".to_string(),
                account: account_id.to_string(),
                arn: identity_arn,
            });

        Ok(AmiResponse::success(identity))
    }
}
