//! Role Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::RoleStore;
use crate::types::PaginationParams;
use crate::wami::identity::Role;
use async_trait::async_trait;

#[async_trait]
impl RoleStore for InMemoryWamiStore {
    async fn create_role(&mut self, role: Role) -> Result<Role> {
        self.roles.insert(role.role_name.clone(), role.clone());
        Ok(role)
    }

    async fn get_role(&self, role_name: &str) -> Result<Option<Role>> {
        Ok(self.roles.get(role_name).cloned())
    }

    async fn update_role(&mut self, role: Role) -> Result<Role> {
        self.roles.insert(role.role_name.clone(), role.clone());
        Ok(role)
    }

    async fn delete_role(&mut self, role_name: &str) -> Result<()> {
        self.roles.remove(role_name);
        Ok(())
    }

    async fn list_roles(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Role>, bool, Option<String>)> {
        let mut roles: Vec<Role> = self.roles.values().cloned().collect();

        if let Some(prefix) = path_prefix {
            roles.retain(|role| role.path.starts_with(prefix));
        }

        roles.sort_by(|a, b| a.role_name.cmp(&b.role_name));

        let mut is_truncated = false;
        let mut marker = None;

        if let Some(pagination) = pagination {
            if let Some(max_items) = pagination.max_items {
                if roles.len() > max_items as usize {
                    roles.truncate(max_items as usize);
                    is_truncated = true;
                    marker = Some(roles.last().unwrap().role_name.clone());
                }
            }
        }

        Ok((roles, is_truncated, marker))
    }

    // Managed policy attachment methods
    async fn attach_role_policy(&mut self, role_name: &str, policy_arn: &str) -> Result<()> {
        let policies = self
            .role_attached_policies
            .entry(role_name.to_string())
            .or_default();

        if !policies.contains(&policy_arn.to_string()) {
            policies.push(policy_arn.to_string());
        }
        Ok(())
    }

    async fn detach_role_policy(&mut self, role_name: &str, policy_arn: &str) -> Result<()> {
        if let Some(policies) = self.role_attached_policies.get_mut(role_name) {
            policies.retain(|p| p != policy_arn);
        }
        Ok(())
    }

    async fn list_attached_role_policies(&self, role_name: &str) -> Result<Vec<String>> {
        Ok(self
            .role_attached_policies
            .get(role_name)
            .cloned()
            .unwrap_or_default())
    }

    // Inline policy methods
    async fn put_role_policy(
        &mut self,
        role_name: &str,
        policy_name: &str,
        policy_document: String,
    ) -> Result<()> {
        let policies = self
            .role_inline_policies
            .entry(role_name.to_string())
            .or_default();

        policies.insert(policy_name.to_string(), policy_document);
        Ok(())
    }

    async fn get_role_policy(&self, role_name: &str, policy_name: &str) -> Result<Option<String>> {
        Ok(self
            .role_inline_policies
            .get(role_name)
            .and_then(|policies| policies.get(policy_name).cloned()))
    }

    async fn delete_role_policy(&mut self, role_name: &str, policy_name: &str) -> Result<()> {
        if let Some(policies) = self.role_inline_policies.get_mut(role_name) {
            policies.remove(policy_name);
        }
        Ok(())
    }

    async fn list_role_policies(&self, role_name: &str) -> Result<Vec<String>> {
        Ok(self
            .role_inline_policies
            .get(role_name)
            .map(|policies| policies.keys().cloned().collect())
            .unwrap_or_default())
    }
}
