//! Inline Policy Service
//!
//! Service for managing inline policies on users, groups, and roles.

use crate::error::{AmiError, Result};
use crate::store::traits::{GroupStore, RoleStore, UserStore};
use crate::wami::policies::inline::*;
use std::sync::{Arc, RwLock};

/// Service for managing inline policies
pub struct InlinePolicyService<S> {
    store: Arc<RwLock<S>>,
}

impl<S> InlinePolicyService<S>
where
    S: UserStore + GroupStore + RoleStore,
{
    /// Create a new InlinePolicyService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    // User inline policy methods

    /// Put an inline policy on a user
    pub async fn put_user_policy(
        &self,
        request: PutUserPolicyRequest,
    ) -> Result<PutUserPolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify user exists
        store
            .get_user(&request.user_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            })?;

        // Validate policy document is valid JSON
        serde_json::from_str::<serde_json::Value>(&request.policy_document).map_err(|e| {
            AmiError::InvalidParameter {
                message: format!("Invalid policy document JSON: {}", e),
            }
        })?;

        // Put the inline policy
        store
            .put_user_policy(
                &request.user_name,
                &request.policy_name,
                request.policy_document,
            )
            .await?;

        Ok(PutUserPolicyResponse {
            message: format!(
                "Inline policy {} added to user {}",
                request.policy_name, request.user_name
            ),
        })
    }

    /// Get an inline policy from a user
    pub async fn get_user_policy(
        &self,
        request: GetUserPolicyRequest,
    ) -> Result<GetUserPolicyResponse> {
        let store = self.store.read().unwrap();

        // Verify user exists
        store
            .get_user(&request.user_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            })?;

        // Get the inline policy
        let policy_document = store
            .get_user_policy(&request.user_name, &request.policy_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!(
                    "Policy {} for user {}",
                    request.policy_name, request.user_name
                ),
            })?;

        Ok(GetUserPolicyResponse {
            user_name: request.user_name,
            policy_name: request.policy_name,
            policy_document,
        })
    }

    /// Delete an inline policy from a user
    pub async fn delete_user_policy(
        &self,
        request: DeleteUserPolicyRequest,
    ) -> Result<DeleteUserPolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify user exists
        store
            .get_user(&request.user_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            })?;

        // Delete the inline policy
        store
            .delete_user_policy(&request.user_name, &request.policy_name)
            .await?;

        Ok(DeleteUserPolicyResponse {
            message: format!(
                "Inline policy {} deleted from user {}",
                request.policy_name, request.user_name
            ),
        })
    }

    /// List inline policies for a user
    pub async fn list_user_policies(
        &self,
        request: ListUserPoliciesRequest,
    ) -> Result<ListUserPoliciesResponse> {
        let store = self.store.read().unwrap();

        // Verify user exists
        store
            .get_user(&request.user_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            })?;

        // List the inline policies
        let policy_names = store.list_user_policies(&request.user_name).await?;

        Ok(ListUserPoliciesResponse { policy_names })
    }

    // Group inline policy methods

    /// Put an inline policy on a group
    pub async fn put_group_policy(
        &self,
        request: PutGroupPolicyRequest,
    ) -> Result<PutGroupPolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify group exists
        store
            .get_group(&request.group_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })?;

        // Validate policy document is valid JSON
        serde_json::from_str::<serde_json::Value>(&request.policy_document).map_err(|e| {
            AmiError::InvalidParameter {
                message: format!("Invalid policy document JSON: {}", e),
            }
        })?;

        // Put the inline policy
        store
            .put_group_policy(
                &request.group_name,
                &request.policy_name,
                request.policy_document,
            )
            .await?;

        Ok(PutGroupPolicyResponse {
            message: format!(
                "Inline policy {} added to group {}",
                request.policy_name, request.group_name
            ),
        })
    }

    /// Get an inline policy from a group
    pub async fn get_group_policy(
        &self,
        request: GetGroupPolicyRequest,
    ) -> Result<GetGroupPolicyResponse> {
        let store = self.store.read().unwrap();

        // Verify group exists
        store
            .get_group(&request.group_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })?;

        // Get the inline policy
        let policy_document = store
            .get_group_policy(&request.group_name, &request.policy_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!(
                    "Policy {} for group {}",
                    request.policy_name, request.group_name
                ),
            })?;

        Ok(GetGroupPolicyResponse {
            group_name: request.group_name,
            policy_name: request.policy_name,
            policy_document,
        })
    }

    /// Delete an inline policy from a group
    pub async fn delete_group_policy(
        &self,
        request: DeleteGroupPolicyRequest,
    ) -> Result<DeleteGroupPolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify group exists
        store
            .get_group(&request.group_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })?;

        // Delete the inline policy
        store
            .delete_group_policy(&request.group_name, &request.policy_name)
            .await?;

        Ok(DeleteGroupPolicyResponse {
            message: format!(
                "Inline policy {} deleted from group {}",
                request.policy_name, request.group_name
            ),
        })
    }

    /// List inline policies for a group
    pub async fn list_group_policies(
        &self,
        request: ListGroupPoliciesRequest,
    ) -> Result<ListGroupPoliciesResponse> {
        let store = self.store.read().unwrap();

        // Verify group exists
        store
            .get_group(&request.group_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })?;

        // List the inline policies
        let policy_names = store.list_group_policies(&request.group_name).await?;

        Ok(ListGroupPoliciesResponse { policy_names })
    }

    // Role inline policy methods

    /// Put an inline policy on a role
    pub async fn put_role_policy(
        &self,
        request: PutRolePolicyRequest,
    ) -> Result<PutRolePolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify role exists
        store
            .get_role(&request.role_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Role: {}", request.role_name),
            })?;

        // Validate policy document is valid JSON
        serde_json::from_str::<serde_json::Value>(&request.policy_document).map_err(|e| {
            AmiError::InvalidParameter {
                message: format!("Invalid policy document JSON: {}", e),
            }
        })?;

        // Put the inline policy
        store
            .put_role_policy(
                &request.role_name,
                &request.policy_name,
                request.policy_document,
            )
            .await?;

        Ok(PutRolePolicyResponse {
            message: format!(
                "Inline policy {} added to role {}",
                request.policy_name, request.role_name
            ),
        })
    }

    /// Get an inline policy from a role
    pub async fn get_role_policy(
        &self,
        request: GetRolePolicyRequest,
    ) -> Result<GetRolePolicyResponse> {
        let store = self.store.read().unwrap();

        // Verify role exists
        store
            .get_role(&request.role_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Role: {}", request.role_name),
            })?;

        // Get the inline policy
        let policy_document = store
            .get_role_policy(&request.role_name, &request.policy_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!(
                    "Policy {} for role {}",
                    request.policy_name, request.role_name
                ),
            })?;

        Ok(GetRolePolicyResponse {
            role_name: request.role_name,
            policy_name: request.policy_name,
            policy_document,
        })
    }

    /// Delete an inline policy from a role
    pub async fn delete_role_policy(
        &self,
        request: DeleteRolePolicyRequest,
    ) -> Result<DeleteRolePolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify role exists
        store
            .get_role(&request.role_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Role: {}", request.role_name),
            })?;

        // Delete the inline policy
        store
            .delete_role_policy(&request.role_name, &request.policy_name)
            .await?;

        Ok(DeleteRolePolicyResponse {
            message: format!(
                "Inline policy {} deleted from role {}",
                request.policy_name, request.role_name
            ),
        })
    }

    /// List inline policies for a role
    pub async fn list_role_policies(
        &self,
        request: ListRolePoliciesRequest,
    ) -> Result<ListRolePoliciesResponse> {
        let store = self.store.read().unwrap();

        // Verify role exists
        store
            .get_role(&request.role_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Role: {}", request.role_name),
            })?;

        // List the inline policies
        let policy_names = store.list_role_policies(&request.role_name).await?;

        Ok(ListRolePoliciesResponse { policy_names })
    }
}
