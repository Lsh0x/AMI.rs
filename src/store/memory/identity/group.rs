//! Group Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::GroupStore;
use crate::types::PaginationParams;
use crate::wami::identity::Group;
use async_trait::async_trait;

#[async_trait]
impl GroupStore for InMemoryWamiStore {
    async fn create_group(&mut self, group: Group) -> Result<Group> {
        self.groups.insert(group.group_name.clone(), group.clone());
        Ok(group)
    }

    async fn get_group(&self, group_name: &str) -> Result<Option<Group>> {
        Ok(self.groups.get(group_name).cloned())
    }

    async fn update_group(&mut self, group: Group) -> Result<Group> {
        self.groups.insert(group.group_name.clone(), group.clone());
        Ok(group)
    }

    async fn delete_group(&mut self, group_name: &str) -> Result<()> {
        self.groups.remove(group_name);
        // Remove from all user-group mappings
        for groups in self.user_groups.values_mut() {
            groups.retain(|g| g != group_name);
        }
        Ok(())
    }

    async fn list_groups(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Group>, bool, Option<String>)> {
        let mut groups: Vec<Group> = self.groups.values().cloned().collect();

        if let Some(prefix) = path_prefix {
            groups.retain(|group| group.path.starts_with(prefix));
        }

        groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));

        let mut is_truncated = false;
        let mut marker = None;

        if let Some(pagination) = pagination {
            if let Some(max_items) = pagination.max_items {
                if groups.len() > max_items as usize {
                    groups.truncate(max_items as usize);
                    is_truncated = true;
                    marker = Some(groups.last().unwrap().group_name.clone());
                }
            }
        }

        Ok((groups, is_truncated, marker))
    }

    async fn list_groups_for_user(&self, user_name: &str) -> Result<Vec<Group>> {
        let group_names = self.user_groups.get(user_name).cloned().unwrap_or_default();
        let groups: Vec<Group> = group_names
            .into_iter()
            .filter_map(|name| self.groups.get(&name).cloned())
            .collect();
        Ok(groups)
    }

    async fn add_user_to_group(&mut self, group_name: &str, user_name: &str) -> Result<()> {
        self.user_groups
            .entry(user_name.to_string())
            .or_default()
            .push(group_name.to_string());
        Ok(())
    }

    async fn remove_user_from_group(&mut self, group_name: &str, user_name: &str) -> Result<()> {
        if let Some(groups) = self.user_groups.get_mut(user_name) {
            groups.retain(|g| g != group_name);
        }
        Ok(())
    }

    // Managed policy attachment methods
    async fn attach_group_policy(&mut self, group_name: &str, policy_arn: &str) -> Result<()> {
        let policies = self
            .group_attached_policies
            .entry(group_name.to_string())
            .or_default();

        if !policies.contains(&policy_arn.to_string()) {
            policies.push(policy_arn.to_string());
        }
        Ok(())
    }

    async fn detach_group_policy(&mut self, group_name: &str, policy_arn: &str) -> Result<()> {
        if let Some(policies) = self.group_attached_policies.get_mut(group_name) {
            policies.retain(|p| p != policy_arn);
        }
        Ok(())
    }

    async fn list_attached_group_policies(&self, group_name: &str) -> Result<Vec<String>> {
        Ok(self
            .group_attached_policies
            .get(group_name)
            .cloned()
            .unwrap_or_default())
    }

    // Inline policy methods
    async fn put_group_policy(
        &mut self,
        group_name: &str,
        policy_name: &str,
        policy_document: String,
    ) -> Result<()> {
        let policies = self
            .group_inline_policies
            .entry(group_name.to_string())
            .or_default();

        policies.insert(policy_name.to_string(), policy_document);
        Ok(())
    }

    async fn get_group_policy(
        &self,
        group_name: &str,
        policy_name: &str,
    ) -> Result<Option<String>> {
        Ok(self
            .group_inline_policies
            .get(group_name)
            .and_then(|policies| policies.get(policy_name).cloned()))
    }

    async fn delete_group_policy(&mut self, group_name: &str, policy_name: &str) -> Result<()> {
        if let Some(policies) = self.group_inline_policies.get_mut(group_name) {
            policies.remove(policy_name);
        }
        Ok(())
    }

    async fn list_group_policies(&self, group_name: &str) -> Result<Vec<String>> {
        Ok(self
            .group_inline_policies
            .get(group_name)
            .map(|policies| policies.keys().cloned().collect())
            .unwrap_or_default())
    }
}
