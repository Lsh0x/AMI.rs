//! Account Assignment Store Trait

use crate::error::Result;
use crate::wami::sso_admin::AccountAssignment;
use async_trait::async_trait;

/// Trait for account assignment storage operations
#[async_trait]
pub trait AccountAssignmentStore: Send + Sync {
    async fn create_account_assignment(
        &mut self,
        assignment: AccountAssignment,
    ) -> Result<AccountAssignment>;

    async fn get_account_assignment(
        &self,
        assignment_id: &str,
    ) -> Result<Option<AccountAssignment>>;

    async fn delete_account_assignment(&mut self, assignment_id: &str) -> Result<()>;

    async fn list_account_assignments(
        &self,
        account_id: &str,
        permission_set_arn: &str,
    ) -> Result<Vec<AccountAssignment>>;
}
