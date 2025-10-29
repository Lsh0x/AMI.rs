//! Permission Set Store Implementation for InMemorySsoAdminStore

use crate::error::Result;
use crate::store::memory::sso_admin::InMemorySsoAdminStore;
use crate::store::traits::PermissionSetStore;
use crate::wami::sso_admin::PermissionSet;
use async_trait::async_trait;

#[async_trait]
impl PermissionSetStore for InMemorySsoAdminStore {
    async fn create_permission_set(
        &mut self,
        permission_set: PermissionSet,
    ) -> Result<PermissionSet> {
        self.permission_sets.insert(
            permission_set.permission_set_arn.clone(),
            permission_set.clone(),
        );
        Ok(permission_set)
    }

    async fn get_permission_set(&self, permission_set_arn: &str) -> Result<Option<PermissionSet>> {
        Ok(self.permission_sets.get(permission_set_arn).cloned())
    }

    async fn update_permission_set(
        &mut self,
        permission_set: PermissionSet,
    ) -> Result<PermissionSet> {
        self.permission_sets.insert(
            permission_set.permission_set_arn.clone(),
            permission_set.clone(),
        );
        Ok(permission_set)
    }

    async fn delete_permission_set(&mut self, permission_set_arn: &str) -> Result<()> {
        self.permission_sets.remove(permission_set_arn);
        Ok(())
    }

    async fn list_permission_sets(&self, _instance_arn: &str) -> Result<Vec<PermissionSet>> {
        Ok(self.permission_sets.values().cloned().collect())
    }
}
