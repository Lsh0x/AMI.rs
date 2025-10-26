//! In-memory SSO Admin Store Implementation

use crate::error::Result;
use crate::sso_admin::{
    AccountAssignment, Application, PermissionSet, SsoInstance, TrustedTokenIssuer,
};
use crate::store::traits::SsoAdminStore;
use async_trait::async_trait;
use std::collections::HashMap;

/// In-memory implementation of SSO Admin store
#[derive(Debug, Default, Clone)]
pub struct InMemorySsoAdminStore {
    permission_sets: HashMap<String, PermissionSet>,
    account_assignments: HashMap<String, AccountAssignment>,
    instances: HashMap<String, SsoInstance>,
    applications: HashMap<String, Application>,
    trusted_token_issuers: HashMap<String, TrustedTokenIssuer>,
}

#[async_trait]
impl SsoAdminStore for InMemorySsoAdminStore {
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

    async fn create_instance(&mut self, instance: SsoInstance) -> Result<SsoInstance> {
        self.instances
            .insert(instance.instance_arn.clone(), instance.clone());
        Ok(instance)
    }

    async fn get_instance(&self, instance_arn: &str) -> Result<Option<SsoInstance>> {
        Ok(self.instances.get(instance_arn).cloned())
    }

    async fn list_instances(&self) -> Result<Vec<SsoInstance>> {
        Ok(self.instances.values().cloned().collect())
    }

    async fn create_application(&mut self, application: Application) -> Result<Application> {
        self.applications
            .insert(application.application_arn.clone(), application.clone());
        Ok(application)
    }

    async fn get_application(&self, application_arn: &str) -> Result<Option<Application>> {
        Ok(self.applications.get(application_arn).cloned())
    }

    async fn list_applications(&self, _instance_arn: &str) -> Result<Vec<Application>> {
        Ok(self.applications.values().cloned().collect())
    }

    async fn create_trusted_token_issuer(
        &mut self,
        issuer: TrustedTokenIssuer,
    ) -> Result<TrustedTokenIssuer> {
        self.trusted_token_issuers
            .insert(issuer.trusted_token_issuer_arn.clone(), issuer.clone());
        Ok(issuer)
    }

    async fn get_trusted_token_issuer(
        &self,
        issuer_arn: &str,
    ) -> Result<Option<TrustedTokenIssuer>> {
        Ok(self.trusted_token_issuers.get(issuer_arn).cloned())
    }

    async fn delete_trusted_token_issuer(&mut self, issuer_arn: &str) -> Result<()> {
        self.trusted_token_issuers.remove(issuer_arn);
        Ok(())
    }

    async fn list_trusted_token_issuers(
        &self,
        _instance_arn: &str,
    ) -> Result<Vec<TrustedTokenIssuer>> {
        Ok(self.trusted_token_issuers.values().cloned().collect())
    }
}
