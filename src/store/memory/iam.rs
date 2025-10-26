use crate::error::Result;
use crate::iam::{AccessKey, Group, LoginProfile, MfaDevice, Policy, Role, User};
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::IamStore;
use crate::types::{PaginationParams, Tag};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// In-memory implementation of IAM store
#[derive(Debug, Clone)]
pub struct InMemoryIamStore {
    account_id: String,
    provider: Arc<dyn CloudProvider>,
    users: HashMap<String, User>,
    access_keys: HashMap<String, AccessKey>,
    groups: HashMap<String, Group>,
    roles: HashMap<String, Role>,
    policies: HashMap<String, Policy>,
    mfa_devices: HashMap<String, MfaDevice>,
    login_profiles: HashMap<String, LoginProfile>,
    user_groups: HashMap<String, Vec<String>>, // user_name -> group_names
    credential_report: Option<crate::iam::reports::CredentialReport>,
    server_certificates: HashMap<String, crate::iam::ServerCertificate>, // cert_name -> certificate
    service_specific_credentials:
        HashMap<String, crate::iam::service_credentials::ServiceSpecificCredential>, // cred_id -> credential
    service_linked_role_deletion_tasks:
        HashMap<String, crate::iam::service_linked_roles::DeletionTaskInfo>, // task_id -> task info
    signing_certificates: HashMap<String, crate::iam::signing_certificates::SigningCertificate>, // cert_id -> certificate
}

impl Default for InMemoryIamStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryIamStore {
    pub fn new() -> Self {
        Self::with_provider(Arc::new(AwsProvider::new()))
    }

    pub fn with_account_id(account_id: String) -> Self {
        Self::with_account_and_provider(account_id, Arc::new(AwsProvider::new()))
    }

    /// Creates a new in-memory IAM store with a custom provider
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::store::memory::InMemoryIamStore;
    /// use wami::provider::{AwsProvider, GcpProvider};
    /// use std::sync::Arc;
    ///
    /// // AWS provider
    /// let aws_store = InMemoryIamStore::with_provider(Arc::new(AwsProvider::new()));
    ///
    /// // GCP provider
    /// let gcp_store = InMemoryIamStore::with_provider(Arc::new(GcpProvider::new("my-project")));
    /// ```
    pub fn with_provider(provider: Arc<dyn CloudProvider>) -> Self {
        Self::with_account_and_provider(crate::types::AwsConfig::generate_account_id(), provider)
    }

    /// Creates a new in-memory IAM store with a specific account ID and provider
    pub fn with_account_and_provider(account_id: String, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            account_id,
            provider,
            users: HashMap::new(),
            access_keys: HashMap::new(),
            groups: HashMap::new(),
            roles: HashMap::new(),
            policies: HashMap::new(),
            mfa_devices: HashMap::new(),
            login_profiles: HashMap::new(),
            user_groups: HashMap::new(),
            credential_report: None,
            server_certificates: HashMap::new(),
            service_specific_credentials: HashMap::new(),
            service_linked_role_deletion_tasks: HashMap::new(),
            signing_certificates: HashMap::new(),
        }
    }
}

#[async_trait]
impl IamStore for InMemoryIamStore {
    fn account_id(&self) -> &str {
        &self.account_id
    }

    fn cloud_provider(&self) -> &dyn CloudProvider {
        self.provider.as_ref()
    }

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

    async fn untag_user(&mut self, user_name: &str, tag_keys: Vec<String>) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_name) {
            user.tags.retain(|tag| !tag_keys.contains(&tag.key));
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

    async fn tag_group(&mut self, group_name: &str, tags: Vec<Tag>) -> Result<()> {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.tags.extend(tags);
        }
        Ok(())
    }

    async fn untag_group(&mut self, group_name: &str, tag_keys: Vec<String>) -> Result<()> {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.tags.retain(|tag| !tag_keys.contains(&tag.key));
        }
        Ok(())
    }

    async fn list_group_tags(&self, group_name: &str) -> Result<Vec<Tag>> {
        Ok(self
            .groups
            .get(group_name)
            .map(|g| g.tags.clone())
            .unwrap_or_default())
    }

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

    async fn tag_role(&mut self, role_name: &str, tags: Vec<Tag>) -> Result<()> {
        if let Some(role) = self.roles.get_mut(role_name) {
            role.tags.extend(tags);
        }
        Ok(())
    }

    async fn untag_role(&mut self, role_name: &str, tag_keys: Vec<String>) -> Result<()> {
        if let Some(role) = self.roles.get_mut(role_name) {
            role.tags.retain(|tag| !tag_keys.contains(&tag.key));
        }
        Ok(())
    }

    async fn list_role_tags(&self, role_name: &str) -> Result<Vec<Tag>> {
        Ok(self
            .roles
            .get(role_name)
            .map(|r| r.tags.clone())
            .unwrap_or_default())
    }

    async fn create_policy(&mut self, policy: Policy) -> Result<Policy> {
        self.policies.insert(policy.arn.clone(), policy.clone());
        Ok(policy)
    }

    async fn get_policy(&self, policy_arn: &str) -> Result<Option<Policy>> {
        Ok(self.policies.get(policy_arn).cloned())
    }

    async fn update_policy(&mut self, policy: Policy) -> Result<Policy> {
        self.policies.insert(policy.arn.clone(), policy.clone());
        Ok(policy)
    }

    async fn delete_policy(&mut self, policy_arn: &str) -> Result<()> {
        self.policies.remove(policy_arn);
        Ok(())
    }

    async fn list_policies(
        &self,
        scope: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Policy>, bool, Option<String>)> {
        let mut policies: Vec<Policy> = self.policies.values().cloned().collect();

        if let Some(scope) = scope {
            policies.retain(|policy| policy.path.starts_with(scope));
        }

        policies.sort_by(|a, b| a.policy_name.cmp(&b.policy_name));

        let mut is_truncated = false;
        let mut marker = None;

        if let Some(pagination) = pagination {
            if let Some(max_items) = pagination.max_items {
                if policies.len() > max_items as usize {
                    policies.truncate(max_items as usize);
                    is_truncated = true;
                    marker = Some(policies.last().unwrap().policy_name.clone());
                }
            }
        }

        Ok((policies, is_truncated, marker))
    }

    async fn tag_policy(&mut self, policy_arn: &str, tags: Vec<Tag>) -> Result<()> {
        if let Some(policy) = self.policies.get_mut(policy_arn) {
            policy.tags.extend(tags);
        }
        Ok(())
    }

    async fn untag_policy(&mut self, policy_arn: &str, tag_keys: Vec<String>) -> Result<()> {
        if let Some(policy) = self.policies.get_mut(policy_arn) {
            policy.tags.retain(|tag| !tag_keys.contains(&tag.key));
        }
        Ok(())
    }

    async fn list_policy_tags(&self, policy_arn: &str) -> Result<Vec<Tag>> {
        Ok(self
            .policies
            .get(policy_arn)
            .map(|p| p.tags.clone())
            .unwrap_or_default())
    }

    async fn create_mfa_device(&mut self, mfa_device: MfaDevice) -> Result<MfaDevice> {
        self.mfa_devices
            .insert(mfa_device.serial_number.clone(), mfa_device.clone());
        Ok(mfa_device)
    }

    async fn get_mfa_device(&self, serial_number: &str) -> Result<Option<MfaDevice>> {
        Ok(self.mfa_devices.get(serial_number).cloned())
    }

    async fn delete_mfa_device(&mut self, serial_number: &str) -> Result<()> {
        self.mfa_devices.remove(serial_number);
        Ok(())
    }

    async fn list_mfa_devices(&self, user_name: &str) -> Result<Vec<MfaDevice>> {
        let devices: Vec<MfaDevice> = self
            .mfa_devices
            .values()
            .filter(|device| device.user_name == user_name)
            .cloned()
            .collect();
        Ok(devices)
    }

    async fn create_login_profile(&mut self, profile: LoginProfile) -> Result<LoginProfile> {
        self.login_profiles
            .insert(profile.user_name.clone(), profile.clone());
        Ok(profile)
    }

    async fn get_login_profile(&self, user_name: &str) -> Result<Option<LoginProfile>> {
        Ok(self.login_profiles.get(user_name).cloned())
    }

    async fn update_login_profile(&mut self, profile: LoginProfile) -> Result<LoginProfile> {
        self.login_profiles
            .insert(profile.user_name.clone(), profile.clone());
        Ok(profile)
    }

    async fn delete_login_profile(&mut self, user_name: &str) -> Result<()> {
        self.login_profiles.remove(user_name);
        Ok(())
    }

    async fn store_credential_report(
        &mut self,
        report: crate::iam::reports::CredentialReport,
    ) -> Result<()> {
        self.credential_report = Some(report);
        Ok(())
    }

    async fn get_credential_report(&self) -> Result<Option<crate::iam::reports::CredentialReport>> {
        Ok(self.credential_report.clone())
    }

    async fn create_server_certificate(
        &mut self,
        certificate: crate::iam::ServerCertificate,
    ) -> Result<crate::iam::ServerCertificate> {
        self.server_certificates.insert(
            certificate
                .server_certificate_metadata
                .server_certificate_name
                .clone(),
            certificate.clone(),
        );
        Ok(certificate)
    }

    async fn get_server_certificate(
        &self,
        certificate_name: &str,
    ) -> Result<Option<crate::iam::ServerCertificate>> {
        Ok(self.server_certificates.get(certificate_name).cloned())
    }

    async fn update_server_certificate(
        &mut self,
        certificate: crate::iam::ServerCertificate,
    ) -> Result<crate::iam::ServerCertificate> {
        self.server_certificates.insert(
            certificate
                .server_certificate_metadata
                .server_certificate_name
                .clone(),
            certificate.clone(),
        );
        Ok(certificate)
    }

    async fn delete_server_certificate(&mut self, certificate_name: &str) -> Result<()> {
        self.server_certificates.remove(certificate_name);
        Ok(())
    }

    async fn list_server_certificates(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<crate::iam::ServerCertificate>, bool, Option<String>)> {
        let mut certificates: Vec<crate::iam::ServerCertificate> =
            self.server_certificates.values().cloned().collect();

        // Apply path prefix filter
        if let Some(prefix) = path_prefix {
            certificates.retain(|cert| cert.server_certificate_metadata.path.starts_with(prefix));
        }

        // Sort by certificate name
        certificates.sort_by(|a, b| {
            a.server_certificate_metadata
                .server_certificate_name
                .cmp(&b.server_certificate_metadata.server_certificate_name)
        });

        // Apply pagination
        let mut is_truncated = false;
        let mut marker = None;

        if let Some(pagination) = pagination {
            if let Some(max_items) = pagination.max_items {
                if certificates.len() > max_items as usize {
                    certificates.truncate(max_items as usize);
                    is_truncated = true;
                    marker = Some(
                        certificates
                            .last()
                            .unwrap()
                            .server_certificate_metadata
                            .server_certificate_name
                            .clone(),
                    );
                }
            }
        }

        Ok((certificates, is_truncated, marker))
    }

    async fn create_service_specific_credential(
        &mut self,
        credential: crate::iam::service_credentials::ServiceSpecificCredential,
    ) -> Result<crate::iam::service_credentials::ServiceSpecificCredential> {
        self.service_specific_credentials.insert(
            credential.service_specific_credential_id.clone(),
            credential.clone(),
        );
        Ok(credential)
    }

    async fn get_service_specific_credential(
        &self,
        credential_id: &str,
    ) -> Result<Option<crate::iam::service_credentials::ServiceSpecificCredential>> {
        Ok(self
            .service_specific_credentials
            .get(credential_id)
            .cloned())
    }

    async fn update_service_specific_credential(
        &mut self,
        credential: crate::iam::service_credentials::ServiceSpecificCredential,
    ) -> Result<crate::iam::service_credentials::ServiceSpecificCredential> {
        self.service_specific_credentials.insert(
            credential.service_specific_credential_id.clone(),
            credential.clone(),
        );
        Ok(credential)
    }

    async fn delete_service_specific_credential(&mut self, credential_id: &str) -> Result<()> {
        self.service_specific_credentials.remove(credential_id);
        Ok(())
    }

    async fn list_service_specific_credentials(
        &self,
        user_name: Option<&str>,
        service_name: Option<&str>,
    ) -> Result<Vec<crate::iam::service_credentials::ServiceSpecificCredential>> {
        let mut credentials: Vec<crate::iam::service_credentials::ServiceSpecificCredential> = self
            .service_specific_credentials
            .values()
            .cloned()
            .collect();

        // Filter by user if provided
        if let Some(user) = user_name {
            credentials.retain(|c| c.user_name == user);
        }

        // Filter by service if provided
        if let Some(service) = service_name {
            credentials.retain(|c| c.service_name == service);
        }

        // Sort by credential ID
        credentials.sort_by(|a, b| {
            a.service_specific_credential_id
                .cmp(&b.service_specific_credential_id)
        });

        Ok(credentials)
    }

    // Service-linked role deletion task operations
    async fn create_service_linked_role_deletion_task(
        &mut self,
        task: crate::iam::service_linked_roles::DeletionTaskInfo,
    ) -> Result<crate::iam::service_linked_roles::DeletionTaskInfo> {
        self.service_linked_role_deletion_tasks
            .insert(task.deletion_task_id.clone(), task.clone());
        Ok(task)
    }

    async fn get_service_linked_role_deletion_task(
        &self,
        task_id: &str,
    ) -> Result<crate::iam::service_linked_roles::DeletionTaskInfo> {
        self.service_linked_role_deletion_tasks
            .get(task_id)
            .cloned()
            .ok_or_else(|| crate::error::AmiError::ResourceNotFound {
                resource: format!("Deletion task {} not found", task_id),
            })
    }

    async fn update_service_linked_role_deletion_task(
        &mut self,
        task: crate::iam::service_linked_roles::DeletionTaskInfo,
    ) -> Result<crate::iam::service_linked_roles::DeletionTaskInfo> {
        self.service_linked_role_deletion_tasks
            .insert(task.deletion_task_id.clone(), task.clone());
        Ok(task)
    }

    // Signing certificate operations
    async fn create_signing_certificate(
        &mut self,
        certificate: crate::iam::signing_certificates::SigningCertificate,
    ) -> Result<crate::iam::signing_certificates::SigningCertificate> {
        self.signing_certificates
            .insert(certificate.certificate_id.clone(), certificate.clone());
        Ok(certificate)
    }

    async fn get_signing_certificate(
        &self,
        certificate_id: &str,
    ) -> Result<Option<crate::iam::signing_certificates::SigningCertificate>> {
        Ok(self.signing_certificates.get(certificate_id).cloned())
    }

    async fn update_signing_certificate(
        &mut self,
        certificate: crate::iam::signing_certificates::SigningCertificate,
    ) -> Result<crate::iam::signing_certificates::SigningCertificate> {
        self.signing_certificates
            .insert(certificate.certificate_id.clone(), certificate.clone());
        Ok(certificate)
    }

    async fn delete_signing_certificate(&mut self, certificate_id: &str) -> Result<()> {
        self.signing_certificates.remove(certificate_id);
        Ok(())
    }

    async fn list_signing_certificates(
        &self,
        user_name: Option<&str>,
    ) -> Result<Vec<crate::iam::signing_certificates::SigningCertificate>> {
        let mut certificates: Vec<crate::iam::signing_certificates::SigningCertificate> =
            self.signing_certificates.values().cloned().collect();

        // Filter by user if provided
        if let Some(user) = user_name {
            certificates.retain(|c| c.user_name == user);
        }

        // Sort by certificate ID
        certificates.sort_by(|a, b| a.certificate_id.cmp(&b.certificate_id));

        Ok(certificates)
    }
}
