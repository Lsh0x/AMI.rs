use crate::error::Result;
use crate::types::{AmiResponse, PaginationParams};
use crate::iam::{IamClient, AccessKey};
use serde::{Deserialize, Serialize};

/// Parameters for creating access keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccessKeyRequest {
    pub user_name: String,
}

/// Parameters for updating access keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccessKeyRequest {
    pub user_name: String,
    pub access_key_id: String,
    pub status: String, // Active, Inactive
}

/// Parameters for listing access keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccessKeysRequest {
    pub user_name: String,
    pub pagination: Option<PaginationParams>,
}

/// Response for listing access keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccessKeysResponse {
    pub access_keys: Vec<AccessKey>,
    pub is_truncated: bool,
    pub marker: Option<String>,
}

/// Response for access key last used information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKeyLastUsed {
    pub last_used_date: Option<chrono::DateTime<chrono::Utc>>,
    pub region: Option<String>,
    pub service_name: Option<String>,
}

impl IamClient {
    /// Create access keys for a user
    pub async fn create_access_key(&mut self, request: CreateAccessKeyRequest) -> Result<AmiResponse<AccessKey>> {
        // Check if user exists
        if !self.users.contains_key(&request.user_name) {
            return Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("User: {}", request.user_name) 
            });
        }

        let access_key_id = format!("AKIA{}", uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(17).collect::<String>());
        let secret_access_key = format!("{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        
        let access_key = AccessKey {
            user_name: request.user_name.clone(),
            access_key_id: access_key_id.clone(),
            status: "Active".to_string(),
            create_date: chrono::Utc::now(),
            secret_access_key: Some(secret_access_key),
        };
        
        self.access_keys.insert(access_key_id, access_key.clone());
        
        Ok(AmiResponse::success(access_key))
    }

    /// Delete access keys
    pub async fn delete_access_key(&mut self, user_name: String, access_key_id: String) -> Result<AmiResponse<()>> {
        if let Some(key) = self.access_keys.get(&access_key_id) {
            if key.user_name == user_name {
                self.access_keys.remove(&access_key_id);
                Ok(AmiResponse::success(()))
            } else {
                Err(crate::error::AmiError::InvalidParameter { 
                    message: "Access key does not belong to the specified user".to_string() 
                })
            }
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("AccessKey: {}", access_key_id) 
            })
        }
    }

    /// Update access key status
    pub async fn update_access_key(&mut self, request: UpdateAccessKeyRequest) -> Result<AmiResponse<AccessKey>> {
        if let Some(key) = self.access_keys.get_mut(&request.access_key_id) {
            if key.user_name == request.user_name {
                key.status = request.status.clone();
                Ok(AmiResponse::success(key.clone()))
            } else {
                Err(crate::error::AmiError::InvalidParameter { 
                    message: "Access key does not belong to the specified user".to_string() 
                })
            }
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("AccessKey: {}", request.access_key_id) 
            })
        }
    }

    /// List access keys for a user
    pub async fn list_access_keys(&self, request: ListAccessKeysRequest) -> Result<AmiResponse<ListAccessKeysResponse>> {
        let mut access_keys: Vec<AccessKey> = self.access_keys
            .values()
            .filter(|key| key.user_name == request.user_name)
            .cloned()
            .collect();
        
        // Sort by access key id
        access_keys.sort_by(|a, b| a.access_key_id.cmp(&b.access_key_id));
        
        // Apply pagination
        let mut is_truncated = false;
        let mut marker = None;
        
        if let Some(pagination) = &request.pagination {
            if let Some(max_items) = pagination.max_items {
                if access_keys.len() > max_items as usize {
                    access_keys.truncate(max_items as usize);
                    is_truncated = true;
                    marker = Some(access_keys.last().unwrap().access_key_id.clone());
                }
            }
        }
        
        let response = ListAccessKeysResponse {
            access_keys,
            is_truncated,
            marker,
        };
        
        Ok(AmiResponse::success(response))
    }

    /// Get access key last used information
    pub async fn get_access_key_last_used(&self, access_key_id: String) -> Result<AmiResponse<AccessKeyLastUsed>> {
        if let Some(_key) = self.access_keys.get(&access_key_id) {
            // In a real implementation, this would track actual usage
            let last_used = AccessKeyLastUsed {
                last_used_date: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
                region: Some("us-east-1".to_string()),
                service_name: Some("iam".to_string()),
            };
            Ok(AmiResponse::success(last_used))
        } else {
            Err(crate::error::AmiError::ResourceNotFound { 
                resource: format!("AccessKey: {}", access_key_id) 
            })
        }
    }
}
