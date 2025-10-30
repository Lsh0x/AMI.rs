//! Policy Attachment Service
//!
//! Service for attaching and detaching managed policies to/from users, groups, and roles.

use crate::error::{AmiError, Result};
use crate::store::traits::{GroupStore, PolicyStore, RoleStore, UserStore};
use crate::wami::policies::attachment::*;
use std::sync::{Arc, RwLock};

/// Service for managing policy attachments
pub struct AttachmentService<S> {
    store: Arc<RwLock<S>>,
}

impl<S> AttachmentService<S>
where
    S: UserStore + GroupStore + RoleStore + PolicyStore,
{
    /// Create a new AttachmentService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    // User policy attachment methods

    /// Attach a managed policy to a user
    pub async fn attach_user_policy(
        &self,
        request: AttachUserPolicyRequest,
    ) -> Result<AttachUserPolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify user exists
        store
            .get_user(&request.user_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            })?;

        // Verify policy exists
        let policy = store
            .get_policy(&request.policy_arn)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Policy: {}", request.policy_arn),
            })?;

        // Verify policy is attachable
        if !policy.is_attachable {
            return Err(AmiError::InvalidParameter {
                message: format!("Policy {} is not attachable", request.policy_arn),
            });
        }

        // Attach the policy
        store
            .attach_user_policy(&request.user_name, &request.policy_arn)
            .await?;

        // Update policy attachment count
        let mut updated_policy = policy.clone();
        updated_policy.attachment_count += 1;
        store.update_policy(updated_policy).await?;

        Ok(AttachUserPolicyResponse {
            message: format!(
                "Policy {} attached to user {}",
                request.policy_arn, request.user_name
            ),
        })
    }

    /// Detach a managed policy from a user
    pub async fn detach_user_policy(
        &self,
        request: DetachUserPolicyRequest,
    ) -> Result<DetachUserPolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify user exists
        store
            .get_user(&request.user_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            })?;

        // Detach the policy
        store
            .detach_user_policy(&request.user_name, &request.policy_arn)
            .await?;

        // Update policy attachment count
        if let Some(policy) = store.get_policy(&request.policy_arn).await? {
            let mut updated_policy = policy.clone();
            updated_policy.attachment_count = updated_policy.attachment_count.saturating_sub(1);
            store.update_policy(updated_policy).await?;
        }

        Ok(DetachUserPolicyResponse {
            message: format!(
                "Policy {} detached from user {}",
                request.policy_arn, request.user_name
            ),
        })
    }

    /// List attached policies for a user
    pub async fn list_attached_user_policies(
        &self,
        request: ListAttachedUserPoliciesRequest,
    ) -> Result<ListAttachedUserPoliciesResponse> {
        let store = self.store.read().unwrap();

        // Verify user exists
        store
            .get_user(&request.user_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            })?;

        // Get attached policy ARNs
        let policy_arns = store
            .list_attached_user_policies(&request.user_name)
            .await?;

        // Convert ARNs to AttachedPolicy objects
        let mut attached_policies = Vec::new();
        for arn in policy_arns {
            if let Some(policy) = store.get_policy(&arn).await? {
                attached_policies.push(AttachedPolicy {
                    policy_name: policy.policy_name,
                    policy_arn: policy.arn,
                });
            }
        }

        Ok(ListAttachedUserPoliciesResponse { attached_policies })
    }

    // Group policy attachment methods

    /// Attach a managed policy to a group
    pub async fn attach_group_policy(
        &self,
        request: AttachGroupPolicyRequest,
    ) -> Result<AttachGroupPolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify group exists
        store
            .get_group(&request.group_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })?;

        // Verify policy exists
        let policy = store
            .get_policy(&request.policy_arn)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Policy: {}", request.policy_arn),
            })?;

        // Verify policy is attachable
        if !policy.is_attachable {
            return Err(AmiError::InvalidParameter {
                message: format!("Policy {} is not attachable", request.policy_arn),
            });
        }

        // Attach the policy
        store
            .attach_group_policy(&request.group_name, &request.policy_arn)
            .await?;

        // Update policy attachment count
        let mut updated_policy = policy.clone();
        updated_policy.attachment_count += 1;
        store.update_policy(updated_policy).await?;

        Ok(AttachGroupPolicyResponse {
            message: format!(
                "Policy {} attached to group {}",
                request.policy_arn, request.group_name
            ),
        })
    }

    /// Detach a managed policy from a group
    pub async fn detach_group_policy(
        &self,
        request: DetachGroupPolicyRequest,
    ) -> Result<DetachGroupPolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify group exists
        store
            .get_group(&request.group_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })?;

        // Detach the policy
        store
            .detach_group_policy(&request.group_name, &request.policy_arn)
            .await?;

        // Update policy attachment count
        if let Some(policy) = store.get_policy(&request.policy_arn).await? {
            let mut updated_policy = policy.clone();
            updated_policy.attachment_count = updated_policy.attachment_count.saturating_sub(1);
            store.update_policy(updated_policy).await?;
        }

        Ok(DetachGroupPolicyResponse {
            message: format!(
                "Policy {} detached from group {}",
                request.policy_arn, request.group_name
            ),
        })
    }

    /// List attached policies for a group
    pub async fn list_attached_group_policies(
        &self,
        request: ListAttachedGroupPoliciesRequest,
    ) -> Result<ListAttachedGroupPoliciesResponse> {
        let store = self.store.read().unwrap();

        // Verify group exists
        store
            .get_group(&request.group_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Group: {}", request.group_name),
            })?;

        // Get attached policy ARNs
        let policy_arns = store
            .list_attached_group_policies(&request.group_name)
            .await?;

        // Convert ARNs to AttachedPolicy objects
        let mut attached_policies = Vec::new();
        for arn in policy_arns {
            if let Some(policy) = store.get_policy(&arn).await? {
                attached_policies.push(AttachedPolicy {
                    policy_name: policy.policy_name,
                    policy_arn: policy.arn,
                });
            }
        }

        Ok(ListAttachedGroupPoliciesResponse { attached_policies })
    }

    // Role policy attachment methods

    /// Attach a managed policy to a role
    pub async fn attach_role_policy(
        &self,
        request: AttachRolePolicyRequest,
    ) -> Result<AttachRolePolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify role exists
        store
            .get_role(&request.role_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Role: {}", request.role_name),
            })?;

        // Verify policy exists
        let policy = store
            .get_policy(&request.policy_arn)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Policy: {}", request.policy_arn),
            })?;

        // Verify policy is attachable
        if !policy.is_attachable {
            return Err(AmiError::InvalidParameter {
                message: format!("Policy {} is not attachable", request.policy_arn),
            });
        }

        // Attach the policy
        store
            .attach_role_policy(&request.role_name, &request.policy_arn)
            .await?;

        // Update policy attachment count
        let mut updated_policy = policy.clone();
        updated_policy.attachment_count += 1;
        store.update_policy(updated_policy).await?;

        Ok(AttachRolePolicyResponse {
            message: format!(
                "Policy {} attached to role {}",
                request.policy_arn, request.role_name
            ),
        })
    }

    /// Detach a managed policy from a role
    pub async fn detach_role_policy(
        &self,
        request: DetachRolePolicyRequest,
    ) -> Result<DetachRolePolicyResponse> {
        let mut store = self.store.write().unwrap();

        // Verify role exists
        store
            .get_role(&request.role_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Role: {}", request.role_name),
            })?;

        // Detach the policy
        store
            .detach_role_policy(&request.role_name, &request.policy_arn)
            .await?;

        // Update policy attachment count
        if let Some(policy) = store.get_policy(&request.policy_arn).await? {
            let mut updated_policy = policy.clone();
            updated_policy.attachment_count = updated_policy.attachment_count.saturating_sub(1);
            store.update_policy(updated_policy).await?;
        }

        Ok(DetachRolePolicyResponse {
            message: format!(
                "Policy {} detached from role {}",
                request.policy_arn, request.role_name
            ),
        })
    }

    /// List attached policies for a role
    pub async fn list_attached_role_policies(
        &self,
        request: ListAttachedRolePoliciesRequest,
    ) -> Result<ListAttachedRolePoliciesResponse> {
        let store = self.store.read().unwrap();

        // Verify role exists
        store
            .get_role(&request.role_name)
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: format!("Role: {}", request.role_name),
            })?;

        // Get attached policy ARNs
        let policy_arns = store
            .list_attached_role_policies(&request.role_name)
            .await?;

        // Convert ARNs to AttachedPolicy objects
        let mut attached_policies = Vec::new();
        for arn in policy_arns {
            if let Some(policy) = store.get_policy(&arn).await? {
                attached_policies.push(AttachedPolicy {
                    policy_name: policy.policy_name,
                    policy_arn: policy.arn,
                });
            }
        }

        Ok(ListAttachedRolePoliciesResponse { attached_policies })
    }
}
