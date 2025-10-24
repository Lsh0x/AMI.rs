use crate::error::Result;
use crate::types::{AmiResponse, AwsConfig};
use crate::store::{SsoAdminStore, Store};
use serde::{Deserialize, Serialize};

/// Generic SSO Admin client that works with any store implementation
#[derive(Debug)]
pub struct SsoAdminClient<S: Store> {
    store: S,
}

impl<S: Store> SsoAdminClient<S> {
    /// Create a new SSO Admin client with a store
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Get mutable reference to the SSO Admin store
    async fn sso_admin_store(&mut self) -> Result<&mut S::SsoAdminStore> {
        self.store.sso_admin_store().await
    }
}

/// Permission set information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    pub permission_set_arn: String,
    pub name: String,
    pub description: Option<String>,
    pub created_date: chrono::DateTime<chrono::Utc>,
    pub session_duration: Option<String>,
    pub relay_state: Option<String>,
}

/// Account assignment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAssignment {
    pub account_id: String,
    pub permission_set_arn: String,
    pub principal_type: String, // USER, GROUP
    pub principal_id: String,
    pub created_date: chrono::DateTime<chrono::Utc>,
}

/// SSO instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoInstance {
    pub instance_arn: String,
    pub identity_store_id: String,
    pub owner_account_id: String,
    pub created_date: chrono::DateTime<chrono::Utc>,
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub application_arn: String,
    pub name: String,
    pub description: Option<String>,
    pub created_date: chrono::DateTime<chrono::Utc>,
}

/// Trusted token issuer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedTokenIssuer {
    pub trusted_token_issuer_arn: String,
    pub name: String,
    pub issuer_url: String,
    pub created_date: chrono::DateTime<chrono::Utc>,
}

/// Parameters for creating permission set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionSetRequest {
    pub instance_arn: String,
    pub name: String,
    pub description: Option<String>,
    pub session_duration: Option<String>,
    pub relay_state: Option<String>,
}

/// Parameters for creating account assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountAssignmentRequest {
    pub instance_arn: String,
    pub target_id: String, // Account ID
    pub target_type: String, // AWS_ACCOUNT
    pub permission_set_arn: String,
    pub principal_type: String, // USER, GROUP
    pub principal_id: String,
}

impl<S: Store> SsoAdminClient<S> {
    /// Create permission set
    pub async fn create_permission_set(&mut self, request: CreatePermissionSetRequest) -> Result<AmiResponse<PermissionSet>> {
        let permission_set_arn = format!("arn:aws:sso:::permissionSet/ssoins-{}", uuid::Uuid::new_v4());
        
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
    pub async fn update_permission_set(&mut self, instance_arn: String, permission_set_arn: String, name: Option<String>, description: Option<String>) -> Result<AmiResponse<PermissionSet>> {
        let store = self.sso_admin_store().await?;
        
        let mut permission_set = store.get_permission_set(&permission_set_arn).await?
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound { 
                resource: format!("PermissionSet: {}", permission_set_arn) 
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
    pub async fn delete_permission_set(&mut self, instance_arn: String, permission_set_arn: String) -> Result<AmiResponse<()>> {
        let store = self.sso_admin_store().await?;
        store.delete_permission_set(&permission_set_arn).await?;
        Ok(AmiResponse::success(()))
    }

    /// Describe permission set
    pub async fn describe_permission_set(&mut self, instance_arn: String, permission_set_arn: String) -> Result<AmiResponse<PermissionSet>> {
        let store = self.sso_admin_store().await?;
        match store.get_permission_set(&permission_set_arn).await? {
            Some(permission_set) => Ok(AmiResponse::success(permission_set)),
            None => Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("PermissionSet: {}", permission_set_arn) 
            })
        }
    }

    /// List permission sets
    pub async fn list_permission_sets(&mut self, instance_arn: String) -> Result<AmiResponse<Vec<PermissionSet>>> {
        let store = self.sso_admin_store().await?;
        let permission_sets = store.list_permission_sets(&instance_arn).await?;
        Ok(AmiResponse::success(permission_sets))
    }

    /// Create account assignment
    pub async fn create_account_assignment(&mut self, request: CreateAccountAssignmentRequest) -> Result<AmiResponse<AccountAssignment>> {
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
    pub async fn delete_account_assignment(&mut self, instance_arn: String, target_id: String, target_type: String, permission_set_arn: String, principal_type: String, principal_id: String) -> Result<AmiResponse<()>> {
        let store = self.sso_admin_store().await?;
        
        // Find the assignment ID
        let assignment_id = format!("{}-{}-{}", target_id, permission_set_arn, principal_id);
        store.delete_account_assignment(&assignment_id).await?;
        
        Ok(AmiResponse::success(()))
    }

    /// List account assignments
    pub async fn list_account_assignments(&mut self, instance_arn: String, account_id: String, permission_set_arn: String) -> Result<AmiResponse<Vec<AccountAssignment>>> {
        let store = self.sso_admin_store().await?;
        let assignments = store.list_account_assignments(&account_id, &permission_set_arn).await?;
        Ok(AmiResponse::success(assignments))
    }

    /// List instances
    pub async fn list_instances(&mut self) -> Result<AmiResponse<Vec<SsoInstance>>> {
        let store = self.sso_admin_store().await?;
        let instances = store.list_instances().await?;
        Ok(AmiResponse::success(instances))
    }

    /// List applications
    pub async fn list_applications(&mut self, instance_arn: String) -> Result<AmiResponse<Vec<Application>>> {
        let store = self.sso_admin_store().await?;
        let applications = store.list_applications(&instance_arn).await?;
        Ok(AmiResponse::success(applications))
    }

    /// Create trusted token issuer
    pub async fn create_trusted_token_issuer(&mut self, instance_arn: String, name: String, issuer_url: String) -> Result<AmiResponse<TrustedTokenIssuer>> {
        let trusted_token_issuer_arn = format!("arn:aws:sso:::trustedTokenIssuer/ssoins-{}", uuid::Uuid::new_v4());
        
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
    pub async fn delete_trusted_token_issuer(&mut self, instance_arn: String, trusted_token_issuer_arn: String) -> Result<AmiResponse<()>> {
        let store = self.sso_admin_store().await?;
        store.delete_trusted_token_issuer(&trusted_token_issuer_arn).await?;
        Ok(AmiResponse::success(()))
    }

    /// List trusted token issuers
    pub async fn list_trusted_token_issuers(&mut self, instance_arn: String) -> Result<AmiResponse<Vec<TrustedTokenIssuer>>> {
        let store = self.sso_admin_store().await?;
        let issuers = store.list_trusted_token_issuers(&instance_arn).await?;
        Ok(AmiResponse::success(issuers))
    }

    /// Describe trusted token issuer
    pub async fn describe_trusted_token_issuer(&mut self, instance_arn: String, trusted_token_issuer_arn: String) -> Result<AmiResponse<TrustedTokenIssuer>> {
        let store = self.sso_admin_store().await?;
        match store.get_trusted_token_issuer(&trusted_token_issuer_arn).await? {
            Some(issuer) => Ok(AmiResponse::success(issuer)),
            None => Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("TrustedTokenIssuer: {}", trusted_token_issuer_arn) 
            })
        }
    }
}