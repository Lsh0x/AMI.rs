//! Tenant Domain Models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hierarchical tenant identifier using opaque numeric IDs
///
/// Format: Slash-separated u64 segments (e.g., "12345678" or "12345678/87654321")
/// Uses `/` separator to align with AWS ARN conventions where paths use slashes.
///
/// # Example
///
/// ```rust
/// use wami::tenant::TenantId;
///
/// let root = TenantId::root();
/// let child = root.child();
/// assert_eq!(child.depth(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantId {
    /// Numeric segments in hierarchy (e.g., [12345678, 87654321])
    segments: Vec<u64>,
}

/// Generate a cryptographically secure random u64 for tenant ID segment
fn generate_secure_u64() -> u64 {
    let mut bytes = [0u8; 8];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
    u64::from_be_bytes(bytes)
}

impl TenantId {
    /// Generate a new root tenant ID with a random numeric segment
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::tenant::TenantId;
    ///
    /// let root = TenantId::root();
    /// assert_eq!(root.depth(), 0);
    /// ```
    pub fn root() -> Self {
        Self {
            segments: vec![generate_secure_u64()],
        }
    }

    /// Create a child tenant ID by appending a new random numeric segment
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::tenant::TenantId;
    ///
    /// let parent = TenantId::root();
    /// let child = parent.child();
    /// assert_eq!(child.depth(), 1);
    /// ```
    pub fn child(&self) -> Self {
        let mut segments = self.segments.clone();
        segments.push(generate_secure_u64());
        Self { segments }
    }

    /// Get parent tenant ID
    ///
    /// Returns None if this is a root tenant.
    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() <= 1 {
            None
        } else {
            Some(Self {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        }
    }

    /// Get hierarchy depth (0 = root, 1 = first level child, etc.)
    pub fn depth(&self) -> usize {
        self.segments.len().saturating_sub(1)
    }

    /// Get all ancestor tenant IDs (including self)
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::tenant::TenantId;
    ///
    /// let root = TenantId::root();
    /// let child = root.child();
    /// let grandchild = child.child();
    /// let ancestors = grandchild.ancestors();
    /// assert_eq!(ancestors.len(), 3);
    /// ```
    pub fn ancestors(&self) -> Vec<TenantId> {
        let mut ancestors = Vec::new();
        for i in 1..=self.segments.len() {
            ancestors.push(TenantId {
                segments: self.segments[..i].to_vec(),
            });
        }
        ancestors
    }

    /// Check if this tenant is a descendant of another
    pub fn is_descendant_of(&self, other: &TenantId) -> bool {
        if self.segments.len() <= other.segments.len() {
            return false;
        }
        self.segments[..other.segments.len()] == other.segments[..]
    }

    /// Check if this tenant is an ancestor of another
    pub fn is_ancestor_of(&self, other: &TenantId) -> bool {
        other.is_descendant_of(self)
    }

    /// Get the tenant ID as a string (slash-separated numeric segments)
    /// Aligns with AWS ARN conventions where paths use `/` separator.
    pub fn as_str(&self) -> String {
        self.segments
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Parse tenant ID from string (for deserialization)
    ///
    /// Format: slash-separated u64 segments (e.g., "12345678/87654321")
    /// Uses `/` separator to match AWS ARN conventions.
    #[allow(clippy::result_large_err)]
    pub fn from_string(s: &str) -> Result<Self, crate::error::AmiError> {
        if s.is_empty() {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Tenant ID cannot be empty".to_string(),
            });
        }

        let segments: Result<Vec<u64>, _> = s
            .split('/')
            .map(|seg| {
                seg.parse::<u64>()
                    .map_err(|_| crate::error::AmiError::InvalidParameter {
                        message: format!("Invalid tenant ID segment: '{}' (must be a u64)", seg),
                    })
            })
            .collect();

        Ok(Self {
            segments: segments?,
        })
    }

    /// Get the segments as a slice (for internal use)
    pub fn segments(&self) -> &[u64] {
        &self.segments
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Custom serialization: serialize as dot-separated string
impl Serialize for TenantId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

// Custom deserialization: parse from dot-separated string
impl<'de> Deserialize<'de> for TenantId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TenantId::from_string(&s).map_err(serde::de::Error::custom)
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
    ///
    /// Names are stored separately from numeric IDs to enable:
    /// - User-friendly display in UI (map name → numeric ID)
    /// - Name-based lookups within a parent tenant
    /// - No information leakage in ARNs (IDs are opaque)
    ///
    /// Names must be unique within the parent tenant to enable reliable name-to-ID mapping.
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
