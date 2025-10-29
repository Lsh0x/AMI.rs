//! User Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::UserStore;
use crate::types::{PaginationParams, Tag};
use crate::wami::identity::User;
use async_trait::async_trait;

#[async_trait]
impl UserStore for InMemoryWamiStore {
    async fn create_user(&mut self, user: User) -> Result<User> {
        self.users.insert(user.user_name.clone(), user.clone());
        Ok(user)
    }

    async fn get_user(&self, user_name: &str) -> Result<Option<User>> {
        Ok(self.users.get(user_name).cloned())
    }

    async fn update_user(&mut self, user: User) -> Result<User> {
        self.users.insert(user.user_name.clone(), user.clone());
        Ok(user)
    }

    async fn delete_user(&mut self, user_name: &str) -> Result<()> {
        self.users.remove(user_name);
        // Also remove associated access keys
        self.access_keys.retain(|_, key| key.user_name != user_name);
        // Remove from user-groups mapping
        self.user_groups.remove(user_name);
        Ok(())
    }

    async fn list_users(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<User>, bool, Option<String>)> {
        let mut users: Vec<User> = self.users.values().cloned().collect();

        // Apply path prefix filter
        if let Some(prefix) = path_prefix {
            users.retain(|user| user.path.starts_with(prefix));
        }

        // Sort by user name
        users.sort_by(|a, b| a.user_name.cmp(&b.user_name));

        // Apply pagination
        let mut is_truncated = false;
        let mut marker = None;

        if let Some(pagination) = pagination {
            if let Some(max_items) = pagination.max_items {
                if users.len() > max_items as usize {
                    users.truncate(max_items as usize);
                    is_truncated = true;
                    marker = Some(users.last().unwrap().user_name.clone());
                }
            }
        }

        Ok((users, is_truncated, marker))
    }

    async fn tag_user(&mut self, user_name: &str, tags: Vec<Tag>) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_name) {
            user.tags.extend(tags);
        }
        Ok(())
    }

    async fn list_user_tags(&self, user_name: &str) -> Result<Vec<Tag>> {
        Ok(self
            .users
            .get(user_name)
            .map(|u| u.tags.clone())
            .unwrap_or_default())
    }

    async fn untag_user(&mut self, user_name: &str, tag_keys: Vec<String>) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_name) {
            user.tags.retain(|tag| !tag_keys.contains(&tag.key));
        }
        Ok(())
    }
}
