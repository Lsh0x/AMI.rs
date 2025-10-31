//! Account Assignment Service
//!
//! Orchestrates account assignment operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::AccountAssignmentStore;
use crate::wami::sso_admin::account_assignment::AccountAssignment;
use std::sync::{Arc, RwLock};

/// Service for managing account assignments
pub struct AccountAssignmentService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
}

impl<S: AccountAssignmentStore> AccountAssignmentService<S> {
    /// Create a new AccountAssignmentService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
        }
    }

    /// Create a new account assignment
    pub async fn create_account_assignment(
        &self,
        assignment: AccountAssignment,
    ) -> Result<AccountAssignment> {
        self.store
            .write()
            .unwrap()
            .create_account_assignment(assignment)
            .await
    }

    /// Get an account assignment by ID
    pub async fn get_account_assignment(
        &self,
        assignment_id: &str,
    ) -> Result<Option<AccountAssignment>> {
        self.store
            .read()
            .unwrap()
            .get_account_assignment(assignment_id)
            .await
    }

    /// Delete an account assignment
    pub async fn delete_account_assignment(&self, assignment_id: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_account_assignment(assignment_id)
            .await
    }

    /// List account assignments for a permission set in an account
    pub async fn list_account_assignments(
        &self,
        account_id: &str,
        permission_set_arn: &str,
    ) -> Result<Vec<AccountAssignment>> {
        self.store
            .read()
            .unwrap()
            .list_account_assignments(account_id, permission_set_arn)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use chrono::Utc;

    fn setup_service() -> AccountAssignmentService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        AccountAssignmentService::new(store)
    }

    fn create_test_assignment(id: &str, perm_set: &str) -> AccountAssignment {
        AccountAssignment {
            assignment_id: id.to_string(),
            instance_arn: "arn:aws:sso:::instance/test-instance".to_string(),
            account_id: "123456789012".to_string(),
            permission_set_arn: perm_set.to_string(),
            principal_id: format!("user-{}", id),
            principal_type: "USER".to_string(),
            target_id: "123456789012".to_string(),
            target_type: "AWS_ACCOUNT".to_string(),
            created_date: Utc::now(),
            wami_arn: format!("arn:wami:.*:0:wami:123456789012:assignment/{}", id)
                .parse()
                .unwrap(),
            providers: vec![],
        }
    }

    #[tokio::test]
    async fn test_create_and_get_assignment() {
        let service = setup_service();
        let assignment = create_test_assignment("assign1", "ps-1");

        let created = service
            .create_account_assignment(assignment.clone())
            .await
            .unwrap();
        assert_eq!(created.assignment_id, "assign1");

        let retrieved = service
            .get_account_assignment(&assignment.assignment_id)
            .await
            .unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_delete_assignment() {
        let service = setup_service();
        let assignment = create_test_assignment("temp", "ps-1");

        service
            .create_account_assignment(assignment.clone())
            .await
            .unwrap();
        service
            .delete_account_assignment(&assignment.assignment_id)
            .await
            .unwrap();

        let retrieved = service
            .get_account_assignment(&assignment.assignment_id)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_assignments() {
        let service = setup_service();
        let account_id = "123456789012";
        let perm_set_arn = "ps-1";

        service
            .create_account_assignment(create_test_assignment("a1", perm_set_arn))
            .await
            .unwrap();
        service
            .create_account_assignment(create_test_assignment("a2", perm_set_arn))
            .await
            .unwrap();

        let assignments = service
            .list_account_assignments(account_id, perm_set_arn)
            .await
            .unwrap();
        assert_eq!(assignments.len(), 2);
    }
}
