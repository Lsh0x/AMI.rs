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
/// Example: arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755
/// ```
///
/// ## Cloud-Synced Resources:
/// ```text
/// arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{provider}:{provider_account_id}:{resource_type}/{resource_id}
/// Examples:
/// - arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:user/77557755
/// - arn:wami:iam:t1/t2/t3:wami:999888777:gcp:554433221:user/77557755
/// - arn:wami:iam:t1/t2/t3:wami:999888777:scaleway:112233445:user/77557755
/// ```
///
/// # Serialization
///
/// `WamiArn` serializes as a string in JSON for compatibility:
/// ```json
/// "arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755"
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
/// Tenants can be organized in a hierarchy (e.g., parent/child/grandchild).
/// The path is represented as a vector of tenant ID segments.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantPath {
    /// Ordered segments of the tenant hierarchy (e.g., ["t1", "t2", "t3"])
    pub segments: Vec<String>,
}

impl TenantPath {
    /// Creates a new tenant path from segments.
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Creates a tenant path from a single tenant ID.
    pub fn single(tenant_id: impl Into<String>) -> Self {
        Self {
            segments: vec![tenant_id.into()],
        }
    }

    /// Returns the full path as a string joined by '/'.
    pub fn as_string(&self) -> String {
        self.segments.join("/")
    }

    /// Returns the root (first) tenant ID.
    pub fn root(&self) -> Option<&str> {
        self.segments.first().map(|s| s.as_str())
    }

    /// Returns the leaf (last) tenant ID.
    pub fn leaf(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
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
    /// arn:wami:iam:t1/t2/t3:wami:999888777
    /// arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:us-east-1
    /// arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:global
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

    /// Returns the primary (root) tenant ID.
    pub fn primary_tenant(&self) -> Option<&str> {
        self.tenant_path.root()
    }

    /// Returns the leaf tenant ID.
    pub fn leaf_tenant(&self) -> Option<&str> {
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
    /// arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755
    /// matches "arn:wami:iam:t1/t2/t3:wami:999888777"
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
        let path = TenantPath::single("t1");
        assert_eq!(path.segments, vec!["t1"]);
        assert_eq!(path.as_string(), "t1");
        assert_eq!(path.root(), Some("t1"));
        assert_eq!(path.leaf(), Some("t1"));
        assert_eq!(path.depth(), 1);
    }

    #[test]
    fn test_tenant_path_hierarchy() {
        let path = TenantPath::new(vec!["t1".to_string(), "t2".to_string(), "t3".to_string()]);
        assert_eq!(path.as_string(), "t1/t2/t3");
        assert_eq!(path.root(), Some("t1"));
        assert_eq!(path.leaf(), Some("t3"));
        assert_eq!(path.depth(), 3);
    }

    #[test]
    fn test_tenant_path_descendant() {
        let parent = TenantPath::new(vec!["t1".to_string(), "t2".to_string()]);
        let child = TenantPath::new(vec!["t1".to_string(), "t2".to_string(), "t3".to_string()]);
        let other = TenantPath::new(vec!["t1".to_string(), "t4".to_string()]);

        assert!(child.is_descendant_of(&parent));
        assert!(parent.is_ancestor_of(&child));
        assert!(!other.is_descendant_of(&parent));
        assert!(!parent.is_descendant_of(&child));
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
    fn test_wami_arn_native() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![
                "t1".to_string(),
                "t2".to_string(),
                "t3".to_string(),
            ]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: None,
            resource: Resource::new("user", "77557755"),
        };

        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755"
        );
        assert_eq!(arn.prefix(), "arn:wami:iam:t1/t2/t3:wami:999888777");
        assert!(!arn.is_cloud_synced());
        assert_eq!(arn.primary_tenant(), Some("t1"));
        assert_eq!(arn.leaf_tenant(), Some("t3"));
        assert_eq!(arn.full_tenant_path(), "t1/t2/t3");
        assert_eq!(arn.resource_type(), "user");
        assert_eq!(arn.resource_id(), "77557755");
    }

    #[test]
    fn test_wami_arn_cloud_synced() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![
                "t1".to_string(),
                "t2".to_string(),
                "t3".to_string(),
            ]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: Some(CloudMapping::new("aws", "223344556677")),
            resource: Resource::new("user", "77557755"),
        };

        assert_eq!(
            arn.to_string(),
            "arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:global:user/77557755"
        );
        assert_eq!(
            arn.prefix(),
            "arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:global"
        );
        assert!(arn.is_cloud_synced());
        assert_eq!(arn.provider(), Some("aws"));
    }

    #[test]
    fn test_wami_arn_cloud_synced_with_region() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![
                "t1".to_string(),
                "t2".to_string(),
                "t3".to_string(),
            ]),
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
            "arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:us-east-1:user/77557755"
        );
        assert_eq!(
            arn.prefix(),
            "arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:us-east-1"
        );
        assert!(arn.is_cloud_synced());
        assert!(arn.cloud_mapping.as_ref().unwrap().is_regional());
    }

    #[test]
    fn test_matches_prefix() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec!["t1".to_string(), "t2".to_string()]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: None,
            resource: Resource::new("user", "77557755"),
        };

        assert!(arn.matches_prefix("arn:wami:iam:t1/t2:wami:999888777"));
        assert!(arn.matches_prefix("arn:wami:iam:t1/t2"));
        assert!(arn.matches_prefix("arn:wami"));
        assert!(!arn.matches_prefix("arn:wami:sts"));
    }

    #[test]
    fn test_belongs_to_tenant() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::new(vec![
                "t1".to_string(),
                "t2".to_string(),
                "t3".to_string(),
            ]),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: None,
            resource: Resource::new("user", "77557755"),
        };

        let same = TenantPath::new(vec!["t1".to_string(), "t2".to_string(), "t3".to_string()]);
        let parent = TenantPath::new(vec!["t1".to_string(), "t2".to_string()]);
        let root = TenantPath::single("t1");
        let other = TenantPath::single("t4");

        assert!(arn.belongs_to_tenant(&same));
        assert!(arn.belongs_to_tenant(&parent));
        assert!(arn.belongs_to_tenant(&root));
        assert!(!arn.belongs_to_tenant(&other));
    }

    #[test]
    fn test_serialization() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::single("t1"),
            wami_instance_id: "999888777".to_string(),
            cloud_mapping: None,
            resource: Resource::new("user", "77557755"),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&arn).unwrap();
        assert_eq!(json, "\"arn:wami:iam:t1:wami:999888777:user/77557755\"");

        // Deserialize from JSON
        let deserialized: WamiArn = serde_json::from_str(&json).unwrap();
        assert_eq!(arn, deserialized);
    }

    #[test]
    fn test_serialization_cloud_synced() {
        let arn = WamiArn {
            service: Service::Iam,
            tenant_path: TenantPath::single("t1"),
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
            "\"arn:wami:iam:t1:wami:999888777:aws:223344556677:us-east-1:user/77557755\""
        );

        // Deserialize from JSON
        let deserialized: WamiArn = serde_json::from_str(&json).unwrap();
        assert_eq!(arn, deserialized);
    }
}
