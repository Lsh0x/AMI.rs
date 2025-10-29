//! Group Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an IAM group
///
/// A group is a collection of IAM users. Groups let you specify permissions for multiple users.
///
/// # Example
///
/// ```rust
/// use wami::iam::group::Group;
/// use chrono::Utc;
///
/// let group = Group {
///     group_name: "Developers".to_string(),
///     group_id: "AGPACKCEVSQ6C2EXAMPLE".to_string(),
///     arn: "arn:aws:iam::123456789012:group/Developers".to_string(),
///     path: "/engineering/".to_string(),
///     create_date: Utc::now(),
///     tags: vec![],
///     wami_arn: "arn:wami:iam::123456789012:group/Developers".to_string(),
///     providers: vec![],
///     tenant_id: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// The friendly name identifying the group
    pub group_name: String,
    /// The stable and unique identifier for the group
    pub group_id: String,
    /// The Amazon Resource Name (ARN) that identifies the group
    pub arn: String,
    /// The path to the group
    pub path: String,
    /// The date and time when the group was created
    pub create_date: DateTime<Utc>,
    /// A list of tags associated with the group
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
    /// Optional tenant ID for multi-tenant isolation
    pub tenant_id: Option<crate::wami::tenant::TenantId>,
}
