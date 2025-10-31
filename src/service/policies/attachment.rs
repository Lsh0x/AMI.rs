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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::identity::group::builder::build_group;
    use crate::wami::identity::role::builder::build_role;
    use crate::wami::identity::user::builder::build_user;
    use crate::wami::policies::policy::builder::build_policy;
    use std::sync::Arc;

    async fn create_test_context() -> WamiContext {
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(0))
            .caller_arn(
                WamiArn::builder()
                    .service(crate::arn::Service::Iam)
                    .tenant_path(TenantPath::single(0))
                    .wami_instance("123456789012")
                    .resource("user", "admin")
                    .build()
                    .unwrap(),
            )
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_attach_user_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = AttachmentService::new(store.clone());
        let context = create_test_context().await;

        // Create user and policy
        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_user = store.write().unwrap().create_user(user).await.unwrap();

        let policy = build_policy(
            "TestPolicy".to_string(),
            r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();
        let created_policy = store.write().unwrap().create_policy(policy).await.unwrap();

        // Attach policy to user
        let request = AttachUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_arn: created_policy.arn.clone(),
        };
        let response = service.attach_user_policy(request).await.unwrap();
        assert!(response.message.contains("attached"));

        // List attached policies
        let list_request = ListAttachedUserPoliciesRequest {
            user_name: "alice".to_string(),
        };
        let list_response = service
            .list_attached_user_policies(list_request)
            .await
            .unwrap();
        assert_eq!(list_response.attached_policies.len(), 1);
        assert_eq!(
            list_response.attached_policies[0].policy_arn,
            created_policy.arn
        );
    }

    #[tokio::test]
    async fn test_detach_user_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = AttachmentService::new(store.clone());
        let context = create_test_context().await;

        // Create user and policy
        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_user = store.write().unwrap().create_user(user).await.unwrap();

        let policy = build_policy(
            "TestPolicy".to_string(),
            r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();
        let created_policy = store.write().unwrap().create_policy(policy).await.unwrap();

        // Attach policy
        let attach_request = AttachUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_arn: created_policy.arn.clone(),
        };
        service.attach_user_policy(attach_request).await.unwrap();

        // Detach policy
        let detach_request = DetachUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_arn: created_policy.arn.clone(),
        };
        let response = service.detach_user_policy(detach_request).await.unwrap();
        assert!(response.message.contains("detached"));

        // Verify detached
        let list_request = ListAttachedUserPoliciesRequest {
            user_name: "alice".to_string(),
        };
        let list_response = service
            .list_attached_user_policies(list_request)
            .await
            .unwrap();
        assert_eq!(list_response.attached_policies.len(), 0);
    }

    #[tokio::test]
    async fn test_attach_group_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = AttachmentService::new(store.clone());
        let context = create_test_context().await;

        // Create group and policy
        let group = build_group("developers".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_group = store.write().unwrap().create_group(group).await.unwrap();

        let policy = build_policy(
            "TestPolicy".to_string(),
            r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();
        let created_policy = store.write().unwrap().create_policy(policy).await.unwrap();

        // Attach policy to group
        let request = AttachGroupPolicyRequest {
            group_name: "developers".to_string(),
            policy_arn: created_policy.arn.clone(),
        };
        let response = service.attach_group_policy(request).await.unwrap();
        assert!(response.message.contains("attached"));
    }

    #[tokio::test]
    async fn test_attach_role_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = AttachmentService::new(store.clone());
        let context = create_test_context().await;

        // Create role and policy
        let role = build_role(
            "AdminRole".to_string(),
            r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();
        let _created_role = store.write().unwrap().create_role(role).await.unwrap();

        let policy = build_policy(
            "TestPolicy".to_string(),
            r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();
        let created_policy = store.write().unwrap().create_policy(policy).await.unwrap();

        // Attach policy to role
        let request = AttachRolePolicyRequest {
            role_name: "AdminRole".to_string(),
            policy_arn: created_policy.arn.clone(),
        };
        let response = service.attach_role_policy(request).await.unwrap();
        assert!(response.message.contains("attached"));
    }

    #[tokio::test]
    async fn test_attach_policy_user_not_found() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = AttachmentService::new(store.clone());
        let context = create_test_context().await;

        let policy = build_policy(
            "TestPolicy".to_string(),
            r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();
        let created_policy = store.write().unwrap().create_policy(policy).await.unwrap();

        let request = AttachUserPolicyRequest {
            user_name: "nonexistent".to_string(),
            policy_arn: created_policy.arn,
        };
        let result = service.attach_user_policy(request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AmiError::ResourceNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_attach_policy_not_found() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = AttachmentService::new(store.clone());
        let context = create_test_context().await;

        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_user = store.write().unwrap().create_user(user).await.unwrap();

        let request = AttachUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_arn: "arn:wami:.*:0:wami:123456789012:policy/nonexistent".to_string(),
        };
        let result = service.attach_user_policy(request).await;
        assert!(result.is_err());
    }
}
