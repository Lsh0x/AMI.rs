//! AWS Single Sign-On (SSO) Admin Operations
//!
//! This module provides functionality for managing AWS SSO resources including permission sets,
//! account assignments, instances, applications, and trusted token issuers.
//!
//! # Overview
//!
//! The SSO Admin module enables you to:
//!
//! - **Permission Sets**: Create and manage permission sets that define access levels
//! - **Account Assignments**: Assign permission sets to users and groups for specific AWS accounts
//! - **Instances**: Manage SSO instances and their configuration
//! - **Applications**: Manage SSO-enabled applications
//! - **Trusted Token Issuers**: Configure trusted token issuers for federation
//!
//! # Example
//!
//! ```rust
//! use wami::{MemorySsoAdminClient, CreatePermissionSetRequest, CreateAccountAssignmentRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = wami::create_memory_store();
//! let mut sso_client = MemorySsoAdminClient::new(store);
//!
//! // Create a permission set
//! let ps_request = CreatePermissionSetRequest {
//!     instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
//!     name: "DataScientistAccess".to_string(),
//!     description: Some("Permissions for data scientists".to_string()),
//!     session_duration: Some("PT8H".to_string()),
//!     relay_state: None,
//! };
//! let ps_response = sso_client.create_permission_set(ps_request).await?;
//! let permission_set = ps_response.data.unwrap();
//! println!("Created permission set: {}", permission_set.permission_set_arn);
//!
//! // Create an account assignment
//! let assignment_request = CreateAccountAssignmentRequest {
//!     instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
//!     target_id: "123456789012".to_string(),
//!     target_type: "AWS_ACCOUNT".to_string(),
//!     permission_set_arn: permission_set.permission_set_arn,
//!     principal_type: "USER".to_string(),
//!     principal_id: "user-id-12345".to_string(),
//! };
//! let assignment_response = sso_client.create_account_assignment(assignment_request).await?;
//! println!("Created assignment: {:?}", assignment_response.data);
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::store::{SsoAdminStore, Store};
use crate::types::AmiResponse;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// SSO Admin client for managing AWS Single Sign-On resources
///
/// The SSO Admin client provides methods for managing permission sets, account assignments,
/// SSO instances, applications, and trusted token issuers. It works with any store
/// implementation that implements the [`Store`] trait.
///
/// # Type Parameters
///
/// * `S` - The store implementation (e.g., [`InMemoryStore`](crate::store::in_memory::InMemoryStore))
///
/// # Example
///
/// ```rust
/// use wami::MemorySsoAdminClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let store = wami::create_memory_store();
/// let mut sso_client = MemorySsoAdminClient::new(store);
///
/// let instances = sso_client.list_instances().await?;
/// println!("SSO instances: {:?}", instances.data);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SsoAdminClient<S: Store> {
    store: S,
}

impl<S: Store> SsoAdminClient<S> {
    /// Creates a new SSO Admin client with the specified store
    ///
    /// # Arguments
    ///
    /// * `store` - The storage backend for SSO Admin resources
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{SsoAdminClient, InMemoryStore};
    ///
    /// let store = InMemoryStore::new();
    /// let sso_client = SsoAdminClient::new(store);
    /// ```
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Get mutable reference to the SSO Admin store
    async fn sso_admin_store(&mut self) -> Result<&mut S::SsoAdminStore> {
        self.store.sso_admin_store().await
    }
}

/// Represents an SSO permission set
///
/// A permission set defines a collection of permissions that can be assigned to users and groups.
///
/// # Example
///
/// ```rust
/// use wami::PermissionSet;
/// use chrono::Utc;
///
/// let permission_set = PermissionSet {
///     permission_set_arn: "arn:aws:sso:::permissionSet/ssoins-1234/ps-5678".to_string(),
///     name: "ReadOnlyAccess".to_string(),
///     description: Some("Read-only access to resources".to_string()),
///     created_date: Utc::now(),
///     session_duration: Some("PT8H".to_string()),
///     relay_state: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    /// The ARN of the permission set
    pub permission_set_arn: String,
    /// The name of the permission set
    pub name: String,
    /// A description of the permission set
    pub description: Option<String>,
    /// The date and time when the permission set was created
    pub created_date: chrono::DateTime<chrono::Utc>,
    /// The length of time that a user can be signed in (ISO-8601 format)
    pub session_duration: Option<String>,
    /// The relay state URL for the application
    pub relay_state: Option<String>,
}

/// Represents an SSO account assignment
///
/// An account assignment grants a user or group access to an AWS account with specific permissions.
///
/// # Example
///
/// ```rust
/// use wami::AccountAssignment;
/// use chrono::Utc;
///
/// let assignment = AccountAssignment {
///     account_id: "123456789012".to_string(),
///     permission_set_arn: "arn:aws:sso:::permissionSet/ssoins-1234/ps-5678".to_string(),
///     principal_type: "USER".to_string(),
///     principal_id: "user-id-12345".to_string(),
///     created_date: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAssignment {
    /// The AWS account ID
    pub account_id: String,
    /// The ARN of the permission set
    pub permission_set_arn: String,
    /// The type of principal: "USER" or "GROUP"
    pub principal_type: String,
    /// The identifier of the principal (user or group)
    pub principal_id: String,
    /// The date and time when the assignment was created
    pub created_date: chrono::DateTime<chrono::Utc>,
}

/// Represents an SSO instance
///
/// # Example
///
/// ```rust
/// use wami::SsoInstance;
/// use chrono::Utc;
///
/// let instance = SsoInstance {
///     instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
///     identity_store_id: "d-1234567890".to_string(),
///     owner_account_id: "123456789012".to_string(),
///     created_date: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoInstance {
    /// The ARN of the SSO instance
    pub instance_arn: String,
    /// The identifier of the identity store
    pub identity_store_id: String,
    /// The AWS account ID that owns the instance
    pub owner_account_id: String,
    /// The date and time when the instance was created
    pub created_date: chrono::DateTime<chrono::Utc>,
}

/// Represents an SSO application
///
/// # Example
///
/// ```rust
/// use wami::Application;
/// use chrono::Utc;
///
/// let application = Application {
///     application_arn: "arn:aws:sso:::application/ssoins-1234/app-5678".to_string(),
///     name: "MyApp".to_string(),
///     description: Some("My application".to_string()),
///     created_date: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    /// The ARN of the application
    pub application_arn: String,
    /// The name of the application
    pub name: String,
    /// A description of the application
    pub description: Option<String>,
    /// The date and time when the application was created
    pub created_date: chrono::DateTime<chrono::Utc>,
}

/// Represents a trusted token issuer
///
/// # Example
///
/// ```rust
/// use wami::TrustedTokenIssuer;
/// use chrono::Utc;
///
/// let issuer = TrustedTokenIssuer {
///     trusted_token_issuer_arn: "arn:aws:sso:::trustedTokenIssuer/ssoins-1234/tti-5678".to_string(),
///     name: "OktaIssuer".to_string(),
///     issuer_url: "https://okta.example.com".to_string(),
///     created_date: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedTokenIssuer {
    /// The ARN of the trusted token issuer
    pub trusted_token_issuer_arn: String,
    /// The name of the issuer
    pub name: String,
    /// The URL of the issuer
    pub issuer_url: String,
    /// The date and time when the issuer was created
    pub created_date: chrono::DateTime<chrono::Utc>,
}

/// Request to create a permission set
///
/// # Example
///
/// ```rust
/// use wami::CreatePermissionSetRequest;
///
/// let request = CreatePermissionSetRequest {
///     instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
///     name: "PowerUserAccess".to_string(),
///     description: Some("Power user permissions".to_string()),
///     session_duration: Some("PT12H".to_string()),
///     relay_state: Some("https://console.aws.amazon.com".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionSetRequest {
    /// The ARN of the SSO instance
    pub instance_arn: String,
    /// The name of the permission set
    pub name: String,
    /// A description of the permission set
    pub description: Option<String>,
    /// The session duration in ISO-8601 format (e.g., "PT8H" for 8 hours)
    pub session_duration: Option<String>,
    /// The relay state URL for the application
    pub relay_state: Option<String>,
}

/// Request to create an account assignment
///
/// # Example
///
/// ```rust
/// use wami::CreateAccountAssignmentRequest;
///
/// let request = CreateAccountAssignmentRequest {
///     instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
///     target_id: "123456789012".to_string(),
///     target_type: "AWS_ACCOUNT".to_string(),
///     permission_set_arn: "arn:aws:sso:::permissionSet/ssoins-1234/ps-5678".to_string(),
///     principal_type: "USER".to_string(),
///     principal_id: "user-id-12345".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountAssignmentRequest {
    /// The ARN of the SSO instance
    pub instance_arn: String,
    /// The AWS account ID
    pub target_id: String,
    /// The target type (always "AWS_ACCOUNT")
    pub target_type: String,
    /// The ARN of the permission set
    pub permission_set_arn: String,
    /// The type of principal: "USER" or "GROUP"
    pub principal_type: String,
    /// The identifier of the principal (user or group)
    pub principal_id: String,
}

impl<S: Store> SsoAdminClient<S> {
    /// Creates a new permission set
    ///
    /// # Arguments
    ///
    /// * `request` - The permission set creation parameters
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemorySsoAdminClient, CreatePermissionSetRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut sso_client = MemorySsoAdminClient::new(store);
    ///
    /// let request = CreatePermissionSetRequest {
    ///     instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
    ///     name: "DeveloperAccess".to_string(),
    ///     description: Some("Developer permissions".to_string()),
    ///     session_duration: Some("PT8H".to_string()),
    ///     relay_state: None,
    /// };
    ///
    /// let response = sso_client.create_permission_set(request).await?;
    /// let permission_set = response.data.unwrap();
    /// println!("Created: {}", permission_set.permission_set_arn);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_permission_set(
        &mut self,
        request: CreatePermissionSetRequest,
    ) -> Result<AmiResponse<PermissionSet>> {
        let permission_set_arn = format!(
            "arn:aws:sso:::permissionSet/ssoins-{}",
            uuid::Uuid::new_v4()
        );

        let permission_set = PermissionSet {
            permission_set_arn: permission_set_arn.clone(),
            name: request.name.clone(),
            description: request.description,
            created_date: chrono::Utc::now(),
            session_duration: request.session_duration,
            relay_state: request.relay_state,
        };

        let store = self.sso_admin_store().await?;
        let created_permission_set = store.create_permission_set(permission_set).await?;

        Ok(AmiResponse::success(created_permission_set))
    }

    /// Update permission set
    pub async fn update_permission_set(
        &mut self,
        _instance_arn: String,
        permission_set_arn: String,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<AmiResponse<PermissionSet>> {
        let store = self.sso_admin_store().await?;

        let mut permission_set = store
            .get_permission_set(&permission_set_arn)
            .await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("PermissionSet: {}", permission_set_arn),
            })?;

        if let Some(new_name) = name {
            permission_set.name = new_name;
        }
        if let Some(new_description) = description {
            permission_set.description = Some(new_description);
        }

        let updated_permission_set = store.update_permission_set(permission_set).await?;
        Ok(AmiResponse::success(updated_permission_set))
    }

    /// Delete permission set
    pub async fn delete_permission_set(
        &mut self,
        _instance_arn: String,
        permission_set_arn: String,
    ) -> Result<AmiResponse<()>> {
        let store = self.sso_admin_store().await?;
        store.delete_permission_set(&permission_set_arn).await?;
        Ok(AmiResponse::success(()))
    }

    /// Describe permission set
    pub async fn describe_permission_set(
        &mut self,
        _instance_arn: String,
        permission_set_arn: String,
    ) -> Result<AmiResponse<PermissionSet>> {
        let store = self.sso_admin_store().await?;
        match store.get_permission_set(&permission_set_arn).await? {
            Some(permission_set) => Ok(AmiResponse::success(permission_set)),
            None => Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("PermissionSet: {}", permission_set_arn),
            }),
        }
    }

    /// List permission sets
    pub async fn list_permission_sets(
        &mut self,
        instance_arn: String,
    ) -> Result<AmiResponse<Vec<PermissionSet>>> {
        let store = self.sso_admin_store().await?;
        let permission_sets = store.list_permission_sets(&instance_arn).await?;
        Ok(AmiResponse::success(permission_sets))
    }

    /// Creates an account assignment for a user or group
    ///
    /// Grants a user or group access to an AWS account with specific permissions.
    ///
    /// # Arguments
    ///
    /// * `request` - The account assignment parameters
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemorySsoAdminClient, CreateAccountAssignmentRequest, CreatePermissionSetRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut sso_client = MemorySsoAdminClient::new(store);
    ///
    /// // First, create a permission set
    /// let ps_request = CreatePermissionSetRequest {
    ///     instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
    ///     name: "AdminAccess".to_string(),
    ///     description: None,
    ///     session_duration: None,
    ///     relay_state: None,
    /// };
    /// let ps_response = sso_client.create_permission_set(ps_request).await?;
    /// let permission_set_arn = ps_response.data.unwrap().permission_set_arn;
    ///
    /// // Create an account assignment
    /// let request = CreateAccountAssignmentRequest {
    ///     instance_arn: "arn:aws:sso:::instance/ssoins-1234".to_string(),
    ///     target_id: "123456789012".to_string(),
    ///     target_type: "AWS_ACCOUNT".to_string(),
    ///     permission_set_arn,
    ///     principal_type: "USER".to_string(),
    ///     principal_id: "user-id-12345".to_string(),
    /// };
    ///
    /// let response = sso_client.create_account_assignment(request).await?;
    /// println!("Assignment created: {:?}", response.data);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_account_assignment(
        &mut self,
        request: CreateAccountAssignmentRequest,
    ) -> Result<AmiResponse<AccountAssignment>> {
        let assignment = AccountAssignment {
            account_id: request.target_id.clone(),
            permission_set_arn: request.permission_set_arn.clone(),
            principal_type: request.principal_type.clone(),
            principal_id: request.principal_id.clone(),
            created_date: chrono::Utc::now(),
        };

        let store = self.sso_admin_store().await?;
        let created_assignment = store.create_account_assignment(assignment).await?;

        Ok(AmiResponse::success(created_assignment))
    }

    /// Delete account assignment
    pub async fn delete_account_assignment(
        &mut self,
        _instance_arn: String,
        target_id: String,
        _target_type: String,
        permission_set_arn: String,
        _principal_type: String,
        principal_id: String,
    ) -> Result<AmiResponse<()>> {
        let store = self.sso_admin_store().await?;

        // Find the assignment ID
        let assignment_id = format!("{}-{}-{}", target_id, permission_set_arn, principal_id);
        store.delete_account_assignment(&assignment_id).await?;

        Ok(AmiResponse::success(()))
    }

    /// List account assignments
    pub async fn list_account_assignments(
        &mut self,
        _instance_arn: String,
        account_id: String,
        permission_set_arn: String,
    ) -> Result<AmiResponse<Vec<AccountAssignment>>> {
        let store = self.sso_admin_store().await?;
        let assignments = store
            .list_account_assignments(&account_id, &permission_set_arn)
            .await?;
        Ok(AmiResponse::success(assignments))
    }

    /// List instances
    pub async fn list_instances(&mut self) -> Result<AmiResponse<Vec<SsoInstance>>> {
        let store = self.sso_admin_store().await?;
        let instances = store.list_instances().await?;
        Ok(AmiResponse::success(instances))
    }

    /// List applications
    pub async fn list_applications(
        &mut self,
        instance_arn: String,
    ) -> Result<AmiResponse<Vec<Application>>> {
        let store = self.sso_admin_store().await?;
        let applications = store.list_applications(&instance_arn).await?;
        Ok(AmiResponse::success(applications))
    }

    /// Create trusted token issuer
    pub async fn create_trusted_token_issuer(
        &mut self,
        _instance_arn: String,
        name: String,
        issuer_url: String,
    ) -> Result<AmiResponse<TrustedTokenIssuer>> {
        let trusted_token_issuer_arn = format!(
            "arn:aws:sso:::trustedTokenIssuer/ssoins-{}",
            uuid::Uuid::new_v4()
        );

        let issuer = TrustedTokenIssuer {
            trusted_token_issuer_arn: trusted_token_issuer_arn.clone(),
            name: name.clone(),
            issuer_url: issuer_url.clone(),
            created_date: chrono::Utc::now(),
        };

        let store = self.sso_admin_store().await?;
        let created_issuer = store.create_trusted_token_issuer(issuer).await?;

        Ok(AmiResponse::success(created_issuer))
    }

    /// Delete trusted token issuer
    pub async fn delete_trusted_token_issuer(
        &mut self,
        _instance_arn: String,
        trusted_token_issuer_arn: String,
    ) -> Result<AmiResponse<()>> {
        let store = self.sso_admin_store().await?;
        store
            .delete_trusted_token_issuer(&trusted_token_issuer_arn)
            .await?;
        Ok(AmiResponse::success(()))
    }

    /// List trusted token issuers
    pub async fn list_trusted_token_issuers(
        &mut self,
        instance_arn: String,
    ) -> Result<AmiResponse<Vec<TrustedTokenIssuer>>> {
        let store = self.sso_admin_store().await?;
        let issuers = store.list_trusted_token_issuers(&instance_arn).await?;
        Ok(AmiResponse::success(issuers))
    }

    /// Describe trusted token issuer
    pub async fn describe_trusted_token_issuer(
        &mut self,
        _instance_arn: String,
        trusted_token_issuer_arn: String,
    ) -> Result<AmiResponse<TrustedTokenIssuer>> {
        let store = self.sso_admin_store().await?;
        match store
            .get_trusted_token_issuer(&trusted_token_issuer_arn)
            .await?
        {
            Some(issuer) => Ok(AmiResponse::success(issuer)),
            None => Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("TrustedTokenIssuer: {}", trusted_token_issuer_arn),
            }),
        }
    }
}
