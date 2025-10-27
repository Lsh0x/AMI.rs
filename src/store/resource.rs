//! Unified Resource Type for Store Operations
//!
//! Provides a single enum to represent all IAM/STS/Tenant resources,
//! enabling generic store operations like `get(arn)` and `query(pattern)`.

use crate::iam::access_key::AccessKey;
use crate::iam::group::Group;
use crate::iam::login_profile::LoginProfile;
use crate::iam::mfa_device::MfaDevice;
use crate::iam::policy::Policy;
use crate::iam::role::Role;
use crate::iam::server_certificate::ServerCertificate;
use crate::iam::service_credential::ServiceSpecificCredential;
use crate::iam::signing_certificate::SigningCertificate;
use crate::iam::user::User;
// Note: STS and Tenant resources don't have `arn` field yet
// use crate::sts::credentials::Credentials;
// use crate::sts::session::StsSession;
// use crate::tenant::Tenant;
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
///
/// // Store can return any resource type
/// let resource = store.get("arn:wami:iam:tenant-x:user/alice").await?;
///
/// match resource {
///     Some(Resource::User(user)) => println!("Found user: {}", user.user_name),
///     Some(Resource::Role(role)) => println!("Found role: {}", role.role_name),
///     None => println!("Not found"),
///     _ => println!("Other resource type"),
/// }
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
    // TODO: Enable once STS/Tenant models have `arn` field
    // /// STS Session
    // StsSession(StsSession),
    // /// STS Credentials
    // Credentials(Credentials),
    // /// Tenant
    // Tenant(Tenant),
}

impl Resource {
    /// Extracts the ARN from any resource type
    ///
    /// All resources must have an `arn` field for this to work.
    pub fn arn(&self) -> &str {
        match self {
            Resource::User(r) => &r.arn,
            Resource::Role(r) => &r.arn,
            Resource::Policy(r) => &r.arn,
            Resource::Group(r) => &r.arn,
            Resource::AccessKey(r) => &r.wami_arn,
            Resource::MfaDevice(r) => &r.wami_arn,
            Resource::LoginProfile(r) => &r.wami_arn,
            Resource::ServerCertificate(r) => &r.server_certificate_metadata.arn,
            Resource::ServiceCredential(r) => &r.wami_arn,
            Resource::SigningCertificate(r) => &r.wami_arn,
            // Resource::StsSession(r) => &r.arn,
            // Resource::Credentials(r) => &r.arn,
            // Resource::Tenant(r) => &r.arn,
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
            // Resource::StsSession(_) => "sts-session",
            // Resource::Credentials(_) => "credentials",
            // Resource::Tenant(_) => "tenant",
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

    // TODO: Enable once Tenant model has `arn` field
    // /// Tries to downcast to Tenant
    // pub fn as_tenant(&self) -> Option<&Tenant> {
    //     if let Resource::Tenant(t) = self {
    //         Some(t)
    //     } else {
    //         None
    //     }
    // }

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

    // TODO: Enable once Tenant model has `arn` field
    // /// Unwraps to Tenant (panics if not Tenant)
    // pub fn into_tenant(self) -> Tenant {
    //     if let Resource::Tenant(t) = self {
    //         t
    //     } else {
    //         panic!("Expected Tenant, got {}", self.resource_type());
    //     }
    // }
}

#[cfg(test)]
mod tests {
    // Note: Tests will be added once we update the models to have `arn` field
}
