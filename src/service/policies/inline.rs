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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::identity::group::builder::build_group;
    use crate::wami::identity::role::builder::build_role;
    use crate::wami::identity::user::builder::build_user;
    use std::sync::Arc;

    async fn create_test_context() -> WamiContext {
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("root"))
            .caller_arn(
                WamiArn::builder()
                    .service(crate::arn::Service::Iam)
                    .tenant_path(TenantPath::single("root"))
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
    async fn test_put_user_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = InlinePolicyService::new(store.clone());
        let context = create_test_context().await;

        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_user = store.write().unwrap().create_user(user).await.unwrap();

        let request = PutUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_name: "MyInlinePolicy".to_string(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
        };
        let response = service.put_user_policy(request).await.unwrap();
        assert!(response.message.contains("added"));
    }

    #[tokio::test]
    async fn test_get_user_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = InlinePolicyService::new(store.clone());
        let context = create_test_context().await;

        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_user = store.write().unwrap().create_user(user).await.unwrap();

        let put_request = PutUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_name: "MyInlinePolicy".to_string(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
        };
        service.put_user_policy(put_request).await.unwrap();

        let get_request = GetUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_name: "MyInlinePolicy".to_string(),
        };
        let response = service.get_user_policy(get_request).await.unwrap();
        assert_eq!(response.policy_name, "MyInlinePolicy");
        assert!(response.policy_document.contains("Version"));
    }

    #[tokio::test]
    async fn test_delete_user_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = InlinePolicyService::new(store.clone());
        let context = create_test_context().await;

        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_user = store.write().unwrap().create_user(user).await.unwrap();

        let put_request = PutUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_name: "MyInlinePolicy".to_string(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
        };
        service.put_user_policy(put_request).await.unwrap();

        let delete_request = DeleteUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_name: "MyInlinePolicy".to_string(),
        };
        let response = service.delete_user_policy(delete_request).await.unwrap();
        assert!(response.message.contains("deleted"));
    }

    #[tokio::test]
    async fn test_list_user_policies() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = InlinePolicyService::new(store.clone());
        let context = create_test_context().await;

        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_user = store.write().unwrap().create_user(user).await.unwrap();

        let put_request1 = PutUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_name: "Policy1".to_string(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
        };
        service.put_user_policy(put_request1).await.unwrap();

        let put_request2 = PutUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_name: "Policy2".to_string(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
        };
        service.put_user_policy(put_request2).await.unwrap();

        let list_request = ListUserPoliciesRequest {
            user_name: "alice".to_string(),
        };
        let response = service.list_user_policies(list_request).await.unwrap();
        assert_eq!(response.policy_names.len(), 2);
    }

    #[tokio::test]
    async fn test_put_group_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = InlinePolicyService::new(store.clone());
        let context = create_test_context().await;

        let group = build_group("developers".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_group = store.write().unwrap().create_group(group).await.unwrap();

        let request = PutGroupPolicyRequest {
            group_name: "developers".to_string(),
            policy_name: "MyInlinePolicy".to_string(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
        };
        let response = service.put_group_policy(request).await.unwrap();
        assert!(response.message.contains("added"));
    }

    #[tokio::test]
    async fn test_put_role_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = InlinePolicyService::new(store.clone());
        let context = create_test_context().await;

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

        let request = PutRolePolicyRequest {
            role_name: "AdminRole".to_string(),
            policy_name: "MyInlinePolicy".to_string(),
            policy_document: r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
        };
        let response = service.put_role_policy(request).await.unwrap();
        assert!(response.message.contains("added"));
    }

    #[tokio::test]
    async fn test_invalid_json_policy() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::new()));
        let service = InlinePolicyService::new(store.clone());
        let context = create_test_context().await;

        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        let _created_user = store.write().unwrap().create_user(user).await.unwrap();

        let request = PutUserPolicyRequest {
            user_name: "alice".to_string(),
            policy_name: "MyInlinePolicy".to_string(),
            policy_document: "invalid json".to_string(),
        };
        let result = service.put_user_policy(request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AmiError::InvalidParameter { .. }
        ));
    }
}
