//! Policy Domain Model

use crate::arn::WamiArn;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an IAM managed policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// The friendly name identifying the policy
    pub policy_name: String,
    /// The stable and unique identifier for the policy
    pub policy_id: String,
    /// The Amazon Resource Name (ARN) that identifies the policy
    pub arn: String,
    /// The path to the policy
    pub path: String,
    /// The identifier for the default version of the policy
    pub default_version_id: String,
    /// The policy document in JSON format
    pub policy_document: String,
    /// The number of entities (users, groups, and roles) that the policy is attached to
    pub attachment_count: i32,
    /// The number of entities that have the policy set as a permissions boundary
    pub permissions_boundary_usage_count: i32,
    /// Whether the policy can be attached to users, groups, or roles
    pub is_attachable: bool,
    /// A friendly description of the policy
    pub description: Option<String>,
    /// The date and time when the policy was created
    pub create_date: DateTime<Utc>,
    /// The date and time when the policy was last updated
    pub update_date: DateTime<Utc>,
    /// A list of tags associated with the policy
    pub tags: Vec<crate::types::Tag>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: WamiArn,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
    /// Optional tenant ID for multi-tenant isolation
    pub tenant_id: Option<crate::wami::tenant::TenantId>,
}
