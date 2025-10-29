//! Access Key Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::AccessKeyStore;
use crate::types::PaginationParams;
use crate::wami::credentials::AccessKey;
use async_trait::async_trait;

#[async_trait]
impl AccessKeyStore for InMemoryWamiStore {
    async fn create_access_key(&mut self, access_key: AccessKey) -> Result<AccessKey> {
        self.access_keys
            .insert(access_key.access_key_id.clone(), access_key.clone());
        Ok(access_key)
    }

    async fn get_access_key(&self, access_key_id: &str) -> Result<Option<AccessKey>> {
        Ok(self.access_keys.get(access_key_id).cloned())
    }

    async fn update_access_key(&mut self, access_key: AccessKey) -> Result<AccessKey> {
        self.access_keys
            .insert(access_key.access_key_id.clone(), access_key.clone());
        Ok(access_key)
    }

    async fn delete_access_key(&mut self, access_key_id: &str) -> Result<()> {
        self.access_keys.remove(access_key_id);
        Ok(())
    }

    async fn list_access_keys(
        &self,
        user_name: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<AccessKey>, bool, Option<String>)> {
        let mut access_keys: Vec<AccessKey> = self
            .access_keys
            .values()
            .filter(|key| key.user_name == user_name)
            .cloned()
            .collect();

        access_keys.sort_by(|a, b| a.access_key_id.cmp(&b.access_key_id));

        let mut is_truncated = false;
        let mut marker = None;

        if let Some(pagination) = pagination {
            if let Some(max_items) = pagination.max_items {
                if access_keys.len() > max_items as usize {
                    access_keys.truncate(max_items as usize);
                    is_truncated = true;
                    marker = Some(access_keys.last().unwrap().access_key_id.clone());
                }
            }
        }

        Ok((access_keys, is_truncated, marker))
    }
}
