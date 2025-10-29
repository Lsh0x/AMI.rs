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
}
