use crate::error::Result;
use crate::iam::{AccessKey, Group, MfaDevice, Policy, Role, User};
use crate::store::IamStore;
use crate::types::{PaginationParams, Tag};
use async_trait::async_trait;
use std::collections::HashMap;

/// In-memory implementation of IAM store
#[derive(Debug, Clone)]
pub struct InMemoryIamStore {
    account_id: String,
    users: HashMap<String, User>,
    access_keys: HashMap<String, AccessKey>,
    groups: HashMap<String, Group>,
    roles: HashMap<String, Role>,
    policies: HashMap<String, Policy>,
    mfa_devices: HashMap<String, MfaDevice>,
    user_groups: HashMap<String, Vec<String>>, // user_name -> group_names
}

impl Default for InMemoryIamStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryIamStore {
    pub fn new() -> Self {
        Self {
            account_id: crate::types::AwsConfig::generate_account_id(),
            users: HashMap::new(),
            access_keys: HashMap::new(),
            groups: HashMap::new(),
            roles: HashMap::new(),
            policies: HashMap::new(),
            mfa_devices: HashMap::new(),
            user_groups: HashMap::new(),
        }
    }

    pub fn with_account_id(account_id: String) -> Self {
        Self {
            account_id,
            users: HashMap::new(),
            access_keys: HashMap::new(),
            groups: HashMap::new(),
            roles: HashMap::new(),
            policies: HashMap::new(),
            mfa_devices: HashMap::new(),
            user_groups: HashMap::new(),
        }
    }
}

#[async_trait]
impl IamStore for InMemoryIamStore {
    fn account_id(&self) -> &str {
        &self.account_id
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
            .or_insert_with(Vec::new)
            .push(group_name.to_string());
        Ok(())
    }

    async fn remove_user_from_group(&mut self, group_name: &str, user_name: &str) -> Result<()> {
        if let Some(groups) = self.user_groups.get_mut(user_name) {
            groups.retain(|g| g != group_name);
        }
        Ok(())
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
}
