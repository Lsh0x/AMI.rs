//! Unified Resource Type for Store Operations
//!
//! Provides a single enum to represent all IAM/STS/Tenant resources,
//! enabling generic store operations like `get(arn)` and `query(pattern)`.

use crate::wami::credentials::access_key::AccessKey;
use crate::wami::credentials::login_profile::LoginProfile;
use crate::wami::credentials::mfa_device::MfaDevice;
use crate::wami::credentials::server_certificate::ServerCertificate;
use crate::wami::credentials::service_credential::ServiceSpecificCredential;
use crate::wami::credentials::signing_certificate::SigningCertificate;
use crate::wami::identity::group::Group;
use crate::wami::identity::role::Role;
use crate::wami::identity::user::User;
use crate::wami::policies::policy::Policy;
use crate::wami::sts::credentials::Credentials;
use crate::wami::sts::session::StsSession;
use crate::wami::tenant::Tenant;
use serde::{Deserialize, Serialize};

/// Unified resource type for generic store operations
///
/// # Purpose
///
/// This enum allows the store to handle all resource types generically:
/// - `store.get(arn)` can return any resource type
/// - `store.query(pattern)` can match across different resource types
/// - Single HashMap can store all resources indexed by ARN
///
/// # Example
///
/// ```rust
/// use wami::store::resource::Resource;
/// use wami::wami::identity::user::User;
/// use chrono::Utc;
///
/// // Create a user resource
/// let user = User {
///     user_name: "alice".to_string(),
///     user_id: "U123".to_string(),
///     arn: "arn:aws:iam::123456789012:user/alice".to_string(),
///     path: "/".to_string(),
///     create_date: Utc::now(),
///     password_last_used: None,
///     permissions_boundary: None,
///     tags: vec![],
///     wami_arn: "arn:wami:iam:tenant-x:wami:123456789012:user/alice".parse().unwrap(),
///     providers: vec![],
///     tenant_id: None,
/// };
///
/// let resource = Resource::User(user);
/// println!("Resource ARN: {}", resource.arn());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "resource_type", content = "data")]
pub enum Resource {
    /// IAM User
    User(User),
    /// IAM Role
    Role(Role),
    /// IAM Policy
    Policy(Policy),
    /// IAM Group
    Group(Group),
    /// Access Key
    AccessKey(AccessKey),
    /// MFA Device
    MfaDevice(MfaDevice),
    /// Login Profile (Console Password)
    LoginProfile(LoginProfile),
    /// Server Certificate
    ServerCertificate(ServerCertificate),
    /// Service-Specific Credential
    ServiceCredential(ServiceSpecificCredential),
    /// Signing Certificate
    SigningCertificate(SigningCertificate),
    /// STS Session
    StsSession(StsSession),
    /// STS Credentials
    Credentials(Credentials),
    /// Tenant
    Tenant(Tenant),
}

impl Resource {
    /// Extracts the ARN from any resource type
    ///
    /// Returns the cloud provider ARN (AWS-format) as a string reference.
    /// For resources that only have WAMI ARNs, this returns a temporary allocation.
    pub fn arn(&self) -> String {
        match self {
            Resource::User(r) => r.arn.clone(),
            Resource::Role(r) => r.arn.clone(),
            Resource::Policy(r) => r.arn.clone(),
            Resource::Group(r) => r.arn.clone(),
            Resource::AccessKey(r) => r.wami_arn.to_string(),
            Resource::MfaDevice(r) => r.wami_arn.to_string(),
            Resource::LoginProfile(r) => r.wami_arn.to_string(),
            Resource::ServerCertificate(r) => r.server_certificate_metadata.arn.clone(),
            Resource::ServiceCredential(r) => r.wami_arn.to_string(),
            Resource::SigningCertificate(r) => r.wami_arn.to_string(),
            Resource::StsSession(r) => r.arn.clone(),
            Resource::Credentials(r) => r.arn.clone(),
            Resource::Tenant(r) => r.arn.clone(),
        }
    }

    /// Gets the resource type name
    pub fn resource_type(&self) -> &'static str {
        match self {
            Resource::User(_) => "user",
            Resource::Role(_) => "role",
            Resource::Policy(_) => "policy",
            Resource::Group(_) => "group",
            Resource::AccessKey(_) => "access-key",
            Resource::MfaDevice(_) => "mfa-device",
            Resource::LoginProfile(_) => "login-profile",
            Resource::ServerCertificate(_) => "server-certificate",
            Resource::ServiceCredential(_) => "service-credential",
            Resource::SigningCertificate(_) => "signing-certificate",
            Resource::StsSession(_) => "session",
            Resource::Credentials(_) => "credentials",
            Resource::Tenant(_) => "tenant",
        }
    }

    /// Tries to downcast to User
    pub fn as_user(&self) -> Option<&User> {
        if let Resource::User(u) = self {
            Some(u)
        } else {
            None
        }
    }

    /// Tries to downcast to Role
    pub fn as_role(&self) -> Option<&Role> {
        if let Resource::Role(r) = self {
            Some(r)
        } else {
            None
        }
    }

    /// Tries to downcast to Policy
    pub fn as_policy(&self) -> Option<&Policy> {
        if let Resource::Policy(p) = self {
            Some(p)
        } else {
            None
        }
    }

    /// Tries to downcast to Group
    pub fn as_group(&self) -> Option<&Group> {
        if let Resource::Group(g) = self {
            Some(g)
        } else {
            None
        }
    }

    /// Tries to downcast to StsSession
    pub fn as_sts_session(&self) -> Option<&StsSession> {
        if let Resource::StsSession(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Tries to downcast to Credentials
    pub fn as_credentials(&self) -> Option<&Credentials> {
        if let Resource::Credentials(c) = self {
            Some(c)
        } else {
            None
        }
    }

    /// Tries to downcast to Tenant
    pub fn as_tenant(&self) -> Option<&Tenant> {
        if let Resource::Tenant(t) = self {
            Some(t)
        } else {
            None
        }
    }

    /// Unwraps to User (panics if not User)
    pub fn into_user(self) -> User {
        if let Resource::User(u) = self {
            u
        } else {
            panic!("Expected User, got {}", self.resource_type());
        }
    }

    /// Unwraps to Role (panics if not Role)
    pub fn into_role(self) -> Role {
        if let Resource::Role(r) = self {
            r
        } else {
            panic!("Expected Role, got {}", self.resource_type());
        }
    }

    /// Unwraps to Policy (panics if not Policy)
    pub fn into_policy(self) -> Policy {
        if let Resource::Policy(p) = self {
            p
        } else {
            panic!("Expected Policy, got {}", self.resource_type());
        }
    }

    /// Unwraps to Group (panics if not Group)
    pub fn into_group(self) -> Group {
        if let Resource::Group(g) = self {
            g
        } else {
            panic!("Expected Group, got {}", self.resource_type());
        }
    }

    /// Unwraps to StsSession (panics if not StsSession)
    pub fn into_sts_session(self) -> StsSession {
        if let Resource::StsSession(s) = self {
            s
        } else {
            panic!("Expected StsSession, got {}", self.resource_type());
        }
    }

    /// Unwraps to Credentials (panics if not Credentials)
    pub fn into_credentials(self) -> Credentials {
        if let Resource::Credentials(c) = self {
            c
        } else {
            panic!("Expected Credentials, got {}", self.resource_type());
        }
    }

    /// Unwraps to Tenant (panics if not Tenant)
    pub fn into_tenant(self) -> Tenant {
        if let Resource::Tenant(t) = self {
            t
        } else {
            panic!("Expected Tenant, got {}", self.resource_type());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::wami::credentials::access_key::builder as access_key_builder;
    use crate::wami::identity::group::builder as group_builder;
    use crate::wami::identity::role::builder as role_builder;
    use crate::wami::identity::user::builder as user_builder;
    use crate::wami::policies::policy::builder as policy_builder;

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:iam:test:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("test"))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[test]
    fn test_resource_user_arn() {
        let context = test_context();
        let user =
            user_builder::build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();

        let resource = Resource::User(user.clone());

        assert_eq!(resource.arn(), user.arn.clone());
        assert_eq!(resource.resource_type(), "user");
    }

    #[test]
    fn test_resource_role_arn() {
        let context = test_context();
        let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();
        let role = role_builder::build_role(
            "admin-role".to_string(),
            trust_policy,
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let resource = Resource::Role(role.clone());

        assert_eq!(resource.arn(), role.arn.clone());
        assert_eq!(resource.resource_type(), "role");
    }

    #[test]
    fn test_resource_group_arn() {
        let context = test_context();
        let group =
            group_builder::build_group("developers".to_string(), Some("/".to_string()), &context)
                .unwrap();

        let resource = Resource::Group(group.clone());

        assert_eq!(resource.arn(), group.arn.clone());
        assert_eq!(resource.resource_type(), "group");
    }

    #[test]
    fn test_resource_policy_arn() {
        let context = test_context();
        let policy_doc = r#"{"Version":"2012-10-17"}"#.to_string();
        let policy = policy_builder::build_policy(
            "TestPolicy".to_string(),
            policy_doc,
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let resource = Resource::Policy(policy.clone());

        assert_eq!(resource.arn(), policy.arn);
        assert_eq!(resource.resource_type(), "policy");
    }

    #[test]
    fn test_resource_access_key_arn() {
        let context = test_context();
        let access_key =
            access_key_builder::build_access_key("alice".to_string(), &context).unwrap();

        let resource = Resource::AccessKey(access_key.clone());

        assert_eq!(resource.arn(), access_key.wami_arn.to_string());
        assert_eq!(resource.resource_type(), "access-key");
    }

    // TODO: Add tenant test when Tenant::new is available

    #[test]
    fn test_resource_as_user() {
        let context = test_context();
        let user =
            user_builder::build_user("bob".to_string(), Some("/".to_string()), &context).unwrap();

        let resource = Resource::User(user.clone());

        let extracted_user = resource.as_user();
        assert!(extracted_user.is_some());
        assert_eq!(extracted_user.unwrap().user_name, "bob");

        // Try wrong type
        assert!(resource.as_role().is_none());
    }

    #[test]
    fn test_resource_as_role() {
        let context = test_context();
        let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();
        let role = role_builder::build_role(
            "test-role".to_string(),
            trust_policy,
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let resource = Resource::Role(role.clone());

        let extracted_role = resource.as_role();
        assert!(extracted_role.is_some());
        assert_eq!(extracted_role.unwrap().role_name, "test-role");

        // Try wrong type
        assert!(resource.as_user().is_none());
    }

    #[test]
    fn test_resource_as_policy() {
        let context = test_context();
        let policy_doc = r#"{"Version":"2012-10-17"}"#.to_string();
        let policy = policy_builder::build_policy(
            "MyPolicy".to_string(),
            policy_doc,
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let resource = Resource::Policy(policy.clone());

        let extracted_policy = resource.as_policy();
        assert!(extracted_policy.is_some());
        assert_eq!(extracted_policy.unwrap().policy_name, "MyPolicy");

        // Try wrong type
        assert!(resource.as_group().is_none());
    }

    #[test]
    fn test_resource_as_group() {
        let context = test_context();
        let group =
            group_builder::build_group("admins".to_string(), Some("/".to_string()), &context)
                .unwrap();

        let resource = Resource::Group(group.clone());

        let extracted_group = resource.as_group();
        assert!(extracted_group.is_some());
        assert_eq!(extracted_group.unwrap().group_name, "admins");
    }

    #[test]
    fn test_resource_into_user() {
        let context = test_context();
        let user = user_builder::build_user("charlie".to_string(), Some("/".to_string()), &context)
            .unwrap();

        let resource = Resource::User(user.clone());
        let extracted = resource.into_user();

        assert_eq!(extracted.user_name, "charlie");
    }

    #[test]
    fn test_resource_into_role() {
        let context = test_context();
        let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();
        let role = role_builder::build_role(
            "my-role".to_string(),
            trust_policy,
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let resource = Resource::Role(role.clone());
        let extracted = resource.into_role();

        assert_eq!(extracted.role_name, "my-role");
    }

    #[test]
    fn test_resource_into_policy() {
        let context = test_context();
        let policy_doc = r#"{"Version":"2012-10-17"}"#.to_string();
        let policy = policy_builder::build_policy(
            "S3Policy".to_string(),
            policy_doc,
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let resource = Resource::Policy(policy.clone());
        let extracted = resource.into_policy();

        assert_eq!(extracted.policy_name, "S3Policy");
    }

    #[test]
    fn test_resource_into_group() {
        let context = test_context();
        let group =
            group_builder::build_group("ops-team".to_string(), Some("/".to_string()), &context)
                .unwrap();

        let resource = Resource::Group(group.clone());
        let extracted = resource.into_group();

        assert_eq!(extracted.group_name, "ops-team");
    }

    #[test]
    #[should_panic(expected = "Expected User, got role")]
    fn test_resource_into_wrong_type_panics() {
        let context = test_context();
        let trust_policy = r#"{"Version":"2012-10-17"}"#.to_string();
        let role = role_builder::build_role(
            "my-role".to_string(),
            trust_policy,
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();

        let resource = Resource::Role(role);

        // This should panic
        let _ = resource.into_user();
    }

    #[test]
    fn test_resource_type_names() {
        let context = test_context();

        // User
        let user =
            user_builder::build_user("test".to_string(), Some("/".to_string()), &context).unwrap();
        assert_eq!(Resource::User(user).resource_type(), "user");

        // Role
        let role = role_builder::build_role(
            "role".to_string(),
            "{}".to_string(),
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();
        assert_eq!(Resource::Role(role).resource_type(), "role");

        // Policy
        let policy = policy_builder::build_policy(
            "pol".to_string(),
            "{}".to_string(),
            Some("/".to_string()),
            None,
            None,
            &context,
        )
        .unwrap();
        assert_eq!(Resource::Policy(policy).resource_type(), "policy");

        // Group
        let group =
            group_builder::build_group("grp".to_string(), Some("/".to_string()), &context).unwrap();
        assert_eq!(Resource::Group(group).resource_type(), "group");

        // Access Key
        let key = access_key_builder::build_access_key("user".to_string(), &context).unwrap();
        assert_eq!(Resource::AccessKey(key).resource_type(), "access-key");
    }

    #[test]
    fn test_resource_all_downcast_combinations() {
        let context = test_context();
        let user =
            user_builder::build_user("test".to_string(), Some("/".to_string()), &context).unwrap();
        let resource = Resource::User(user);

        // User should only match as_user
        assert!(resource.as_user().is_some());
        assert!(resource.as_role().is_none());
        assert!(resource.as_policy().is_none());
        assert!(resource.as_group().is_none());
        assert!(resource.as_sts_session().is_none());
        assert!(resource.as_credentials().is_none());
        assert!(resource.as_tenant().is_none());
    }
}
