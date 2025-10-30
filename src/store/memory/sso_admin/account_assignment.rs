//! Account Assignment Store Implementation for InMemorySsoAdminStore

use crate::error::Result;
use crate::store::memory::sso_admin::InMemorySsoAdminStore;
use crate::store::traits::AccountAssignmentStore;
use crate::wami::sso_admin::AccountAssignment;
use async_trait::async_trait;

#[async_trait]
impl AccountAssignmentStore for InMemorySsoAdminStore {
    async fn create_account_assignment(
        &mut self,
        assignment: AccountAssignment,
    ) -> Result<AccountAssignment> {
        let assignment_id = format!(
            "{}-{}-{}",
            assignment.account_id, assignment.permission_set_arn, assignment.principal_id
        );
        self.account_assignments
            .insert(assignment_id, assignment.clone());
        Ok(assignment)
    }

    async fn get_account_assignment(
        &self,
        assignment_id: &str,
    ) -> Result<Option<AccountAssignment>> {
        Ok(self.account_assignments.get(assignment_id).cloned())
    }

    async fn delete_account_assignment(&mut self, assignment_id: &str) -> Result<()> {
        self.account_assignments.remove(assignment_id);
        Ok(())
    }

    async fn list_account_assignments(
        &self,
        account_id: &str,
        permission_set_arn: &str,
    ) -> Result<Vec<AccountAssignment>> {
        let assignments: Vec<AccountAssignment> = self
            .account_assignments
            .values()
            .filter(|assignment| {
                assignment.account_id == account_id
                    && assignment.permission_set_arn == permission_set_arn
            })
            .cloned()
            .collect();
        Ok(assignments)
    }
}

/// Implement AccountAssignmentStore for InMemoryWamiStore (the main unified store)
#[async_trait]
impl AccountAssignmentStore for super::super::wami::InMemoryWamiStore {
    async fn create_account_assignment(
        &mut self,
        assignment: AccountAssignment,
    ) -> Result<AccountAssignment> {
        self.account_assignments
            .insert(assignment.assignment_id.clone(), assignment.clone());
        Ok(assignment)
    }

    async fn get_account_assignment(
        &self,
        assignment_id: &str,
    ) -> Result<Option<AccountAssignment>> {
        Ok(self.account_assignments.get(assignment_id).cloned())
    }

    async fn delete_account_assignment(&mut self, assignment_id: &str) -> Result<()> {
        self.account_assignments.remove(assignment_id);
        Ok(())
    }

    async fn list_account_assignments(
        &self,
        account_id: &str,
        permission_set_arn: &str,
    ) -> Result<Vec<AccountAssignment>> {
        Ok(self
            .account_assignments
            .values()
            .filter(|a| a.account_id == account_id && a.permission_set_arn == permission_set_arn)
            .cloned()
            .collect())
    }
}
