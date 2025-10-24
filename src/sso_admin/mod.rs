use crate::error::Result;
use crate::types::{AmiResponse, AwsConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// In-memory SSO Admin client that simulates AWS SSO Admin operations
#[derive(Debug, Clone)]
pub struct SsoAdminClient {
    // In-memory storage for SSO resources
    permission_sets: HashMap<String, PermissionSet>,
    assignments: HashMap<String, AccountAssignment>,
    instances: HashMap<String, SsoInstance>,
    applications: HashMap<String, Application>,
    trusted_token_issuers: HashMap<String, TrustedTokenIssuer>,
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

impl SsoAdminClient {
    /// Create a new in-memory SSO Admin client
    pub async fn new() -> Result<Self> {
        Ok(Self {
            permission_sets: HashMap::new(),
            assignments: HashMap::new(),
            instances: HashMap::new(),
            applications: HashMap::new(),
            trusted_token_issuers: HashMap::new(),
        })
    }

    /// Create a new SSO Admin client with custom configuration
    pub async fn with_config(_config: AwsConfig) -> Result<Self> {
        // For in-memory implementation, config is not used
        Self::new().await
    }

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
        
        self.permission_sets.insert(permission_set_arn, permission_set.clone());
        
        Ok(AmiResponse::success(permission_set))
    }

    /// Update permission set
    pub async fn update_permission_set(&mut self, instance_arn: String, permission_set_arn: String, name: Option<String>, description: Option<String>) -> Result<AmiResponse<PermissionSet>> {
        if let Some(mut permission_set) = self.permission_sets.get_mut(&permission_set_arn) {
            if let Some(new_name) = name {
                permission_set.name = new_name;
            }
            if let Some(new_description) = description {
                permission_set.description = Some(new_description);
            }
            
            Ok(AmiResponse::success(permission_set.clone()))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("PermissionSet: {}", permission_set_arn) 
            })
        }
    }

    /// Delete permission set
    pub async fn delete_permission_set(&mut self, instance_arn: String, permission_set_arn: String) -> Result<AmiResponse<()>> {
        if self.permission_sets.remove(&permission_set_arn).is_some() {
            Ok(AmiResponse::success(()))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("PermissionSet: {}", permission_set_arn) 
            })
        }
    }

    /// Describe permission set
    pub async fn describe_permission_set(&self, instance_arn: String, permission_set_arn: String) -> Result<AmiResponse<PermissionSet>> {
        match self.permission_sets.get(&permission_set_arn) {
            Some(permission_set) => Ok(AmiResponse::success(permission_set.clone())),
            None => Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("PermissionSet: {}", permission_set_arn) 
            })
        }
    }

    /// List permission sets
    pub async fn list_permission_sets(&self, instance_arn: String) -> Result<AmiResponse<Vec<PermissionSet>>> {
        let permission_sets: Vec<PermissionSet> = self.permission_sets.values().cloned().collect();
        Ok(AmiResponse::success(permission_sets))
    }

    /// Create account assignment
    pub async fn create_account_assignment(&mut self, request: CreateAccountAssignmentRequest) -> Result<AmiResponse<AccountAssignment>> {
        let assignment_id = format!("{}", uuid::Uuid::new_v4());
        
        let assignment = AccountAssignment {
            account_id: request.target_id.clone(),
            permission_set_arn: request.permission_set_arn.clone(),
            principal_type: request.principal_type.clone(),
            principal_id: request.principal_id.clone(),
            created_date: chrono::Utc::now(),
        };
        
        self.assignments.insert(assignment_id, assignment.clone());
        
        Ok(AmiResponse::success(assignment))
    }

    /// Delete account assignment
    pub async fn delete_account_assignment(&mut self, instance_arn: String, target_id: String, target_type: String, permission_set_arn: String, principal_type: String, principal_id: String) -> Result<AmiResponse<()>> {
        // Find and remove the assignment
        let assignment_key = self.assignments.iter()
            .find(|(_, assignment)| {
                assignment.account_id == target_id &&
                assignment.permission_set_arn == permission_set_arn &&
                assignment.principal_type == principal_type &&
                assignment.principal_id == principal_id
            })
            .map(|(key, _)| key.clone());
        
        if let Some(key) = assignment_key {
            self.assignments.remove(&key);
            Ok(AmiResponse::success(()))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: "AccountAssignment".to_string() 
            })
        }
    }

    /// List account assignments
    pub async fn list_account_assignments(&self, instance_arn: String, account_id: String, permission_set_arn: String) -> Result<AmiResponse<Vec<AccountAssignment>>> {
        let assignments: Vec<AccountAssignment> = self.assignments
            .values()
            .filter(|assignment| assignment.account_id == account_id && assignment.permission_set_arn == permission_set_arn)
            .cloned()
            .collect();
        
        Ok(AmiResponse::success(assignments))
    }

    /// List instances
    pub async fn list_instances(&self) -> Result<AmiResponse<Vec<SsoInstance>>> {
        let instances: Vec<SsoInstance> = self.instances.values().cloned().collect();
        Ok(AmiResponse::success(instances))
    }

    /// List applications
    pub async fn list_applications(&self, instance_arn: String) -> Result<AmiResponse<Vec<Application>>> {
        let applications: Vec<Application> = self.applications.values().cloned().collect();
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
        
        self.trusted_token_issuers.insert(trusted_token_issuer_arn, issuer.clone());
        
        Ok(AmiResponse::success(issuer))
    }

    /// Delete trusted token issuer
    pub async fn delete_trusted_token_issuer(&mut self, instance_arn: String, trusted_token_issuer_arn: String) -> Result<AmiResponse<()>> {
        if self.trusted_token_issuers.remove(&trusted_token_issuer_arn).is_some() {
            Ok(AmiResponse::success(()))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("TrustedTokenIssuer: {}", trusted_token_issuer_arn) 
            })
        }
    }

    /// List trusted token issuers
    pub async fn list_trusted_token_issuers(&self, instance_arn: String) -> Result<AmiResponse<Vec<TrustedTokenIssuer>>> {
        let issuers: Vec<TrustedTokenIssuer> = self.trusted_token_issuers.values().cloned().collect();
        Ok(AmiResponse::success(issuers))
    }

    /// Describe trusted token issuer
    pub async fn describe_trusted_token_issuer(&self, instance_arn: String, trusted_token_issuer_arn: String) -> Result<AmiResponse<TrustedTokenIssuer>> {
        match self.trusted_token_issuers.get(&trusted_token_issuer_arn) {
            Some(issuer) => Ok(AmiResponse::success(issuer.clone())),
            None => Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("TrustedTokenIssuer: {}", trusted_token_issuer_arn) 
            })
        }
    }
}
