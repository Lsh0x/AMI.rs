//! Tenant Domain Models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hierarchical tenant identifier
///
/// Format: "root" or "root/child" or "root/child/grandchild"
///
/// # Example
///
/// ```rust
/// use wami::tenant::TenantId;
///
/// let root = TenantId::root("acme");
/// let child = root.child("engineering");
/// assert_eq!(child.as_str(), "acme/engineering");
/// assert_eq!(child.depth(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TenantId(String);

impl TenantId {
    /// Create a new tenant ID from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create a root tenant ID
    pub fn root(name: &str) -> Self {
        Self(name.to_string())
    }

    /// Create a child tenant ID
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::tenant::TenantId;
    ///
    /// let parent = TenantId::root("acme");
    /// let child = parent.child("engineering");
    /// assert_eq!(child.as_str(), "acme/engineering");
    /// ```
    pub fn child(&self, name: &str) -> Self {
        Self(format!("{}/{}", self.0, name))
    }

    /// Get parent tenant ID
    ///
    /// Returns None if this is a root tenant.
    pub fn parent(&self) -> Option<Self> {
        self.0
            .rsplit_once('/')
            .map(|(parent, _)| Self(parent.to_string()))
    }

    /// Get hierarchy depth (0 = root, 1 = first level child, etc.)
    pub fn depth(&self) -> usize {
        self.0.matches('/').count()
    }

    /// Get all ancestor tenant IDs (including self)
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::tenant::TenantId;
    ///
    /// let id = TenantId::new("acme/eng/team1");
    /// let ancestors = id.ancestors();
    /// assert_eq!(ancestors.len(), 3);
    /// assert_eq!(ancestors[0].as_str(), "acme");
    /// assert_eq!(ancestors[1].as_str(), "acme/eng");
    /// assert_eq!(ancestors[2].as_str(), "acme/eng/team1");
    /// ```
    pub fn ancestors(&self) -> Vec<TenantId> {
        let mut ancestors = Vec::new();
        let parts: Vec<&str> = self.0.split('/').collect();

        for i in 1..=parts.len() {
            ancestors.push(TenantId(parts[0..i].join("/")));
        }

        ancestors
    }

    /// Check if this tenant is a descendant of another
    pub fn is_descendant_of(&self, other: &TenantId) -> bool {
        self.0.starts_with(&format!("{}/", other.0))
    }

    /// Get the tenant ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tenant entity with hierarchical support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// Unique hierarchical tenant identifier
    pub id: TenantId,

    /// Parent tenant (None for root tenants)
    pub parent_id: Option<TenantId>,

    /// Display name (unique within parent)
    pub name: String,

    /// Organization/company name
    pub organization: Option<String>,

    /// Tenant type
    pub tenant_type: TenantType,

    /// Cloud provider accounts per tenant
    /// Maps: provider_name -> account_id
    pub provider_accounts: HashMap<String, String>,

    /// The WAMI ARN for this tenant (opaque tenant hash)
    /// Format: arn:wami:tenant:global:tenant/tenant-hash
    pub arn: String,

    /// List of cloud providers where this tenant exists
    pub providers: Vec<crate::provider::ProviderConfig>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Tenant status
    pub status: TenantStatus,

    /// Resource quotas
    pub quotas: TenantQuotas,

    /// Quota inheritance mode
    pub quota_mode: QuotaMode,

    /// Maximum depth for sub-tenants
    pub max_child_depth: usize,

    /// Can create sub-tenants
    pub can_create_sub_tenants: bool,

    /// Tenant admin principals (user ARNs)
    pub admin_principals: Vec<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// Billing information
    pub billing_info: Option<BillingInfo>,
}

/// Tenant type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TenantType {
    /// Platform root tenant
    Root,
    /// Enterprise customer
    Enterprise,
    /// Department within enterprise
    Department,
    /// Team within department
    Team,
    /// Project/workspace
    Project,
    /// Custom type
    Custom(String),
}

/// Tenant status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    /// Tenant is active and operational
    Active,
    /// Tenant is suspended (read-only)
    Suspended,
    /// Tenant is pending activation
    Pending,
    /// Tenant is marked for deletion
    Deleted,
}

/// Quota inheritance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaMode {
    /// Inherit from parent
    Inherited,
    /// Override with custom quotas
    Override,
}

/// Resource quotas for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
    /// Maximum IAM users
    pub max_users: usize,
    /// Maximum roles
    pub max_roles: usize,
    /// Maximum policies
    pub max_policies: usize,
    /// Maximum groups
    pub max_groups: usize,
    /// Maximum access keys
    pub max_access_keys: usize,
    /// Maximum sub-tenants
    pub max_sub_tenants: usize,
    /// API rate limit (requests per minute)
    pub api_rate_limit: usize,
}

impl TenantQuotas {
    /// Validate that child quotas don't exceed parent quotas
    pub fn validate_against_parent(&self, parent: &TenantQuotas) -> Result<(), String> {
        if self.max_users > parent.max_users {
            return Err("max_users exceeds parent limit".to_string());
        }
        if self.max_roles > parent.max_roles {
            return Err("max_roles exceeds parent limit".to_string());
        }
        if self.max_policies > parent.max_policies {
            return Err("max_policies exceeds parent limit".to_string());
        }
        if self.max_groups > parent.max_groups {
            return Err("max_groups exceeds parent limit".to_string());
        }
        if self.max_sub_tenants > parent.max_sub_tenants {
            return Err("max_sub_tenants exceeds parent limit".to_string());
        }
        Ok(())
    }
}

impl Default for TenantQuotas {
    fn default() -> Self {
        Self {
            max_users: 1000,
            max_roles: 500,
            max_policies: 100,
            max_groups: 100,
            max_access_keys: 2000,
            max_sub_tenants: 10,
            api_rate_limit: 1000,
        }
    }
}

/// Billing information for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingInfo {
    /// Cost center or billing account
    pub cost_center: String,
    /// Whether this tenant is billable
    pub billable: bool,
    /// Billing contact email
    pub contact_email: String,
}

/// Current resource usage for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsage {
    /// Tenant ID
    pub tenant_id: TenantId,
    /// Current user count
    pub current_users: usize,
    /// Current role count
    pub current_roles: usize,
    /// Current policy count
    pub current_policies: usize,
    /// Current group count
    pub current_groups: usize,
    /// Current sub-tenant count
    pub current_sub_tenants: usize,
    /// Include descendants in count
    pub include_descendants: bool,
}
