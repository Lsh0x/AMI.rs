//! Core ARN types and structures for WAMI's multi-tenant, multi-cloud architecture.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Represents a WAMI ARN (Amazon Resource Name).
///
/// # Format
///
/// ## WAMI Native (no cloud sync):
/// ```text
/// arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{resource_type}/{resource_id}
/// Example: arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/77557755
/// ```
///
/// ## Cloud-Synced Resources:
/// ```text
/// arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{provider}:{provider_account_id}:{resource_type}/{resource_id}
/// Examples:
/// - arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:user/77557755
/// - arn:wami:iam:12345678/87654321/99999999:wami:999888777:gcp:554433221:user/77557755
/// - arn:wami:iam:12345678/87654321/99999999:wami:999888777:scaleway:112233445:user/77557755
/// ```
///
/// # Serialization
///
/// `WamiArn` serializes as a string in JSON for compatibility:
/// ```json
/// "arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/77557755"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WamiArn {
    /// The service this resource belongs to (iam, sts, sso-admin, etc.)
    pub service: Service,

    /// The hierarchical tenant path (e.g., t1/t2/t3)
    pub tenant_path: TenantPath,

    /// The WAMI instance ID (unique identifier for this WAMI deployment)
    pub wami_instance_id: String,

    /// Optional cloud provider mapping for synced resources
    pub cloud_mapping: Option<CloudMapping>,

    /// The resource type and ID
    pub resource: Resource,
}

/// Represents a hierarchical tenant path.
///
/// Tenants can be organized in a hierarchy using opaque numeric IDs.
/// The path is represented as a vector of numeric tenant ID segments.
/// Uses `/` separator to align with AWS ARN conventions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantPath {
    /// Ordered segments of the tenant hierarchy (e.g., [12345678, 87654321, 99999999])
    pub segments: Vec<u64>,
}

impl TenantPath {
    /// Creates a new tenant path from numeric segments.
    pub fn new(segments: Vec<u64>) -> Self {
        Self { segments }
    }

    /// Creates a tenant path from a single numeric tenant ID.
    pub fn single(tenant_id: u64) -> Self {
        Self {
            segments: vec![tenant_id],
        }
    }

    /// Creates a tenant path from a TenantId.
    pub fn from_tenant_id(tenant_id: &crate::wami::tenant::TenantId) -> Self {
        Self {
            segments: tenant_id.segments().to_vec(),
        }
    }

    /// Returns the full path as a string joined by '/'.
    pub fn as_string(&self) -> String {
        self.segments
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Returns the root (first) tenant ID as a string.
    pub fn root(&self) -> Option<String> {
        self.segments.first().map(|n| n.to_string())
    }

    /// Returns the leaf (last) tenant ID as a string.
    pub fn leaf(&self) -> Option<String> {
        self.segments.last().map(|n| n.to_string())
    }

    /// Returns the root (first) tenant ID as u64.
    pub fn root_u64(&self) -> Option<u64> {
        self.segments.first().copied()
    }

    /// Returns the leaf (last) tenant ID as u64.
    pub fn leaf_u64(&self) -> Option<u64> {
        self.segments.last().copied()
    }

    /// Returns the depth of the tenant hierarchy.
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Check if this tenant path starts with another tenant path (is a child or same)
    pub fn starts_with(&self, other: &TenantPath) -> bool {
        if self.segments.len() < other.segments.len() {
            return false;
        }

        for (i, segment) in other.segments.iter().enumerate() {
            if self.segments[i] != *segment {
                return false;
            }
        }

        true
    }

    /// Returns true if this path is a descendant of the given path.
    pub fn is_descendant_of(&self, other: &TenantPath) -> bool {
        if self.segments.len() <= other.segments.len() {
            return false;
        }
        self.segments
            .iter()
            .zip(other.segments.iter())
            .all(|(a, b)| a == b)
    }

    /// Returns true if this path is an ancestor of the given path.
    pub fn is_ancestor_of(&self, other: &TenantPath) -> bool {
        other.is_descendant_of(self)
    }
}

// Custom serialization: serialize as slash-separated string
impl Serialize for TenantPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_string())
    }
}

// Custom deserialization: parse from slash-separated string
impl<'de> Deserialize<'de> for TenantPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            return Err(serde::de::Error::custom("Tenant path cannot be empty"));
        }

        let segments: Result<Vec<u64>, _> = s
            .split('/')
            .map(|seg| {
                seg.parse::<u64>().map_err(|_| {
                    serde::de::Error::custom(format!(
                        "Invalid tenant path segment: '{}' (must be a u64)",
                        seg
                    ))
                })
            })
            .collect();

        Ok(Self {
            segments: segments?,
        })
    }
}

impl fmt::Display for TenantPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Represents a cloud provider mapping for synced resources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CloudMapping {
    /// Cloud provider name (aws, gcp, azure, scaleway, etc.)
    pub provider: String,

    /// Provider-specific account ID
    pub account_id: String,

    /// Optional region/location (us-east-1, us-central1, global, etc.)
    /// None indicates a global resource or region-independent resource
    pub region: Option<String>,
}

impl CloudMapping {
    /// Creates a new cloud mapping without a region (global resource).
    pub fn new(provider: impl Into<String>, account_id: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            account_id: account_id.into(),
            region: None,
        }
    }

    /// Creates a new cloud mapping with a specific region.
    pub fn with_region(
        provider: impl Into<String>,
        account_id: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            account_id: account_id.into(),
            region: Some(region.into()),
        }
    }

    /// Returns true if this mapping has a region specified.
    pub fn is_regional(&self) -> bool {
        self.region.is_some()
    }

    /// Returns the region, or "global" if none is specified.
    pub fn region_or_global(&self) -> &str {
        self.region.as_deref().unwrap_or("global")
    }
}

/// Represents a resource (type and ID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resource {
    /// Resource type (user, role, policy, group, etc.)
    pub resource_type: String,

    /// Resource ID (stable identifier, not the name)
    pub resource_id: String,
}

impl Resource {
    /// Creates a new resource.
    pub fn new(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
        }
    }

    /// Returns the resource path as "type/id".
    pub fn as_path(&self) -> String {
        format!("{}/{}", self.resource_type, self.resource_id)
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_path())
    }
}

/// WAMI service types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    /// IAM (Identity and Access Management)
    Iam,

    /// STS (Security Token Service)
    Sts,

    /// SSO Admin
    #[serde(rename = "sso-admin")]
    SsoAdmin,

    /// Custom service
    Custom(String),
}

impl Service {
    /// Returns the service name as a string.
    pub fn as_str(&self) -> &str {
        match self {
            Service::Iam => "iam",
            Service::Sts => "sts",
            Service::SsoAdmin => "sso-admin",
            Service::Custom(s) => s.as_str(),
        }
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Service {
    fn from(s: &str) -> Self {
        match s {
            "iam" => Service::Iam,
            "sts" => Service::Sts,
            "sso-admin" => Service::SsoAdmin,
            other => Service::Custom(other.to_string()),
        }
    }
}

impl WamiArn {
    /// Returns the ARN prefix (everything before the resource).
    ///
    /// # Examples
    ///
    /// ```text
    /// arn:wami:iam:12345678/87654321/99999999:wami:999888777
    /// arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:us-east-1
    /// arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:global
    /// ```
    pub fn prefix(&self) -> String {
        let base = format!(
            "arn:wami:{}:{}:wami:{}",
            self.service, self.tenant_path, self.wami_instance_id
        );

        if let Some(ref mapping) = self.cloud_mapping {
            let region = mapping.region_or_global();
            format!(
                "{}:{}:{}:{}",
                base, mapping.provider, mapping.account_id, region
            )
        } else {
            base
        }
    }

    /// Returns true if this resource is synced with a cloud provider.
    pub fn is_cloud_synced(&self) -> bool {
        self.cloud_mapping.is_some()
    }

    /// Returns the cloud provider name if synced.
    pub fn provider(&self) -> Option<&str> {
        self.cloud_mapping.as_ref().map(|m| m.provider.as_str())
    }

    /// Returns the primary (root) tenant ID as a string.
    pub fn primary_tenant(&self) -> Option<String> {
        self.tenant_path.root()
    }

    /// Returns the leaf tenant ID as a string.
    pub fn leaf_tenant(&self) -> Option<String> {
        self.tenant_path.leaf()
    }

    /// Returns the full tenant path as a string.
    pub fn full_tenant_path(&self) -> String {
        self.tenant_path.as_string()
    }

    /// Returns true if this ARN matches the given prefix.
    ///
    /// Useful for querying resources by prefix (e.g., all resources in a tenant).
    ///
    /// # Examples
    ///
    /// ```text
    /// arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/77557755
    /// matches "arn:wami:iam:12345678/87654321/99999999:wami:999888777"
    /// ```
    pub fn matches_prefix(&self, prefix: &str) -> bool {
        self.to_string().starts_with(prefix)
    }

    /// Returns true if this ARN belongs to the given tenant path or its descendants.
    pub fn belongs_to_tenant(&self, tenant_path: &TenantPath) -> bool {
        &self.tenant_path == tenant_path || self.tenant_path.is_descendant_of(tenant_path)
    }

    /// Returns the resource type.
    pub fn resource_type(&self) -> &str {
        &self.resource.resource_type
    }

    /// Returns the resource ID.
    pub fn resource_id(&self) -> &str {
        &self.resource.resource_id
    }
}

impl fmt::Display for WamiArn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.prefix(), self.resource)
    }
}

// Custom serialization to serialize WamiArn as a string
impl Serialize for WamiArn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Custom deserialization to deserialize WamiArn from a string
impl<'de> Deserialize<'de> for WamiArn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        WamiArn::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_path_single() {
        let path = TenantPath::single(12345678);
        assert_eq!(path.segments, vec![12345678]);
        assert_eq!(path.as_string(), "12345678");
        assert_eq!(path.root(), Some("12345678".to_string()));
        assert_eq!(path.leaf(), Some("12345678".to_string()));
        assert_eq!(path.depth(), 1);
    }

    #[test]
    fn test_tenant_path_hierarchy() {
        let path = TenantPath::new(vec![12345678, 87654321, 99999999]);
        assert_eq!(path.as_string(), "12345678/87654321/99999999");
        assert_eq!(path.root(), Some("12345678".to_string()));
        assert_eq!(path.leaf(), Some("99999999".to_string()));
        assert_eq!(path.depth(), 3);
    }

    #[test]
    fn test_tenant_path_descendant() {
        let parent = TenantPath::new(vec![12345678, 87654321]);
        let child = TenantPath::new(vec![12345678, 87654321, 99999999]);
        let other = TenantPath::new(vec![12345678, 11111111]);

        assert!(child.is_descendant_of(&parent));
        assert!(parent.is_ancestor_of(&child));
        assert!(!other.is_descendant_of(&parent));
        assert!(!parent.is_descendant_of(&child));
    }

    #[test]
    fn test_tenant_path_starts_with() {
        let path1 = TenantPath::new(vec![12345678, 87654321, 99999999]);
        let path2 = TenantPath::new(vec![12345678, 87654321]);
        let path3 = TenantPath::new(vec![12345678]);
        let path4 = TenantPath::new(vec![12345678, 87654321, 99999999, 11111111]);
        let path5 = TenantPath::new(vec![12345678, 55555555]);

        assert!(path1.starts_with(&path2));
        assert!(path1.starts_with(&path3));
        assert!(path4.starts_with(&path1));
        assert!(path1.starts_with(&path1)); // Same path
        assert!(!path1.starts_with(&path5)); // Different path
        assert!(!path2.starts_with(&path1)); // path2 is shorter
    }

    #[test]
    fn test_tenant_path_empty_root_leaf() {
        let empty_path = TenantPath::new(vec![]);
        assert_eq!(empty_path.root(), None);
        assert_eq!(empty_path.leaf(), None);
        assert_eq!(empty_path.depth(), 0);
        assert_eq!(empty_path.as_string(), "");
    }

    #[test]
    fn test_cloud_mapping_global_vs_regional() {
        let global = CloudMapping::new("aws", "123456789012");
        assert!(!global.is_regional());
        assert_eq!(global.region_or_global(), "global");

        let regional = CloudMapping::with_region("aws", "123456789012", "us-east-1");
        assert!(regional.is_regional());
        assert_eq!(regional.region_or_global(), "us-east-1");
    }

    #[test]
    fn test_wami_arn_matches_prefix() {
        let arn: WamiArn = "arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/alice"
            .parse()
            .unwrap();

        assert!(arn.matches_prefix("arn:wami:iam:12345678/87654321/99999999:wami:999888777"));
        assert!(arn.matches_prefix("arn:wami:iam:12345678/87654321"));
        assert!(arn.matches_prefix("arn:wami:iam:12345678"));
        assert!(arn.matches_prefix("arn:wami:iam"));
        assert!(arn.matches_prefix("arn:wami"));
        assert!(arn.matches_prefix("arn"));
        assert!(!arn.matches_prefix("arn:wami:sts"));
        assert!(!arn.matches_prefix("arn:wami:iam:12345678/87654321/11111111"));
    }

    #[test]
    fn test_wami_arn_belongs_to_tenant() {
        let arn: WamiArn = "arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/alice"
            .parse()
            .unwrap();

        // Exact match
        let tenant = TenantPath::new(vec![12345678, 87654321, 99999999]);
        assert!(arn.belongs_to_tenant(&tenant));

        // Parent tenant (descendant)
        let parent = TenantPath::new(vec![12345678, 87654321]);
        assert!(arn.belongs_to_tenant(&parent));

        let root = TenantPath::single(12345678);
        assert!(arn.belongs_to_tenant(&root));

        // Not related
        let other = TenantPath::single(99999999);
        assert!(!arn.belongs_to_tenant(&other));

        let sibling = TenantPath::new(vec![12345678, 11111111]);
        assert!(!arn.belongs_to_tenant(&sibling));
    }

    #[test]
    fn test_resource() {
        let resource = Resource::new("user", "77557755");
        assert_eq!(resource.as_path(), "user/77557755");
        assert_eq!(resource.to_string(), "user/77557755");
    }

    #[test]
    fn test_service() {
        assert_eq!(Service::Iam.as_str(), "iam");
        assert_eq!(Service::Sts.as_str(), "sts");
        assert_eq!(Service::SsoAdmin.as_str(), "sso-admin");
        assert_eq!(Service::Custom("custom".to_string()).as_str(), "custom");
    }

    #[test]
    fn test_service_from_str() {
        assert_eq!(Service::from("iam"), Service::Iam);
        assert_eq!(Service::from("sts"), Service::Sts);
        assert_eq!(Service::from("sso-admin"), Service::SsoAdmin);
        assert_eq!(
            Service::from("custom"),
            Service::Custom("custom".to_string())
        );
    }

    #[test]
    fn test_tenant_path_from_tenant_id() {
        use crate::wami::tenant::TenantId;

        // Root tenant ID
        let root_id = TenantId::root();
        let root_path = TenantPath::from_tenant_id(&root_id);
        assert_eq!(root_path.segments, root_id.segments().to_vec());
        assert_eq!(root_path.depth(), root_id.segments().len());

        // Child tenant ID
        let child_id = root_id.child();
        let child_path = TenantPath::from_tenant_id(&child_id);
        assert_eq!(child_path.segments, child_id.segments().to_vec());
        assert_eq!(child_path.segments.len(), 2);
        assert_eq!(child_path.depth(), 2);

        // Multi-level hierarchy
        let grandchild_id = child_id.child();
        let grandchild_path = TenantPath::from_tenant_id(&grandchild_id);
        assert_eq!(grandchild_path.segments, grandchild_id.segments().to_vec());
        assert_eq!(grandchild_path.segments.len(), 3);
        assert_eq!(grandchild_path.depth(), 3);
    }

    #[test]
    fn test_wami_arn_native() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![12345678, 87654321, 99999999]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: None,
            resource: Resource::new("user", "77557755"),
        };

        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:user/77557755"
        );
        assert_eq!(
            arn.prefix(),
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777"
        );
        assert!(!arn.is_cloud_synced());
        assert_eq!(arn.primary_tenant(), Some("12345678".to_string()));
        assert_eq!(arn.leaf_tenant(), Some("99999999".to_string()));
        assert_eq!(arn.full_tenant_path(), "12345678/87654321/99999999");
        assert_eq!(arn.resource_type(), "user");
        assert_eq!(arn.resource_id(), "77557755");
    }

    #[test]
    fn test_wami_arn_cloud_synced() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![12345678, 87654321, 99999999]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: Some(CloudMapping::new("aws", "223344556677")),
            resource: Resource::new("user", "77557755"),
        };

        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:global:user/77557755"
        );
        assert_eq!(
            arn.prefix(),
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:global"
        );
        assert!(arn.is_cloud_synced());
        assert_eq!(arn.provider(), Some("aws"));
    }

    #[test]
    fn test_wami_arn_cloud_synced_with_region() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![12345678, 87654321, 99999999]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: Some(CloudMapping::with_region(
                "aws",
                "223344556677",
                "us-east-1",
            )),
            resource: Resource::new("user", "77557755"),
        };

        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:us-east-1:user/77557755"
        );
        assert_eq!(
            arn.prefix(),
            "arn:wami:iam:12345678/87654321/99999999:wami:999888777:aws:223344556677:us-east-1"
        );
        assert!(arn.is_cloud_synced());
        assert!(arn.cloud_mapping.as_ref().unwrap().is_regional());
    }

    #[test]
    fn test_matches_prefix() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![12345678, 87654321]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: None,
            resource: Resource::new("user", "77557755"),
        };

        assert!(arn.matches_prefix("arn:wami:iam:12345678/87654321:wami:999888777"));
        assert!(arn.matches_prefix("arn:wami:iam:12345678/87654321"));
        assert!(arn.matches_prefix("arn:wami"));
        assert!(!arn.matches_prefix("arn:wami:sts"));
    }

    #[test]
    fn test_belongs_to_tenant() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![12345678, 87654321, 99999999]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: None,
            resource: Resource::new("user", "77557755"),
        };

        let same = TenantPath::new(vec![12345678, 87654321, 99999999]);
        let parent = TenantPath::new(vec![12345678, 87654321]);
        let root = TenantPath::single(12345678);
        let other = TenantPath::single(99999999);

        assert!(arn.belongs_to_tenant(&same));
        assert!(arn.belongs_to_tenant(&parent));
        assert!(arn.belongs_to_tenant(&root));
        assert!(!arn.belongs_to_tenant(&other));
    }

    #[test]
    fn test_serialization() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::single(12345678),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: None,
            resource: Resource::new("user", "77557755"),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&arn).unwrap();
        assert_eq!(
            json,
            "\"arn:wami:iam:12345678:wami:999888777:user/77557755\""
        );

        // Deserialize from JSON
        let deserialized: WamiArn = serde_json::from_str(&json).unwrap();
        assert_eq!(arn, deserialized);
    }

    #[test]
    fn test_serialization_cloud_synced() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::single(12345678),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: Some(CloudMapping::with_region(
                "aws",
                "223344556677",
                "us-east-1",
            )),
            resource: Resource::new("user", "77557755"),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&arn).unwrap();
        assert_eq!(
            json,
            "\"arn:wami:iam:12345678:wami:999888777:aws:223344556677:us-east-1:user/77557755\""
        );

        // Deserialize from JSON
        let deserialized: WamiArn = serde_json::from_str(&json).unwrap();
        assert_eq!(arn, deserialized);
    }
}
