//! Identity Domain Model

use crate::arn::WamiArn;
use serde::{Deserialize, Serialize};

/// Information about the caller's identity
///
/// # Example
///
/// ```rust
/// use wami::sts::CallerIdentity;
///
/// let identity = CallerIdentity {
///     user_id: "AIDACKCEVSQ6C2EXAMPLE".to_string(),
///     account: "123456789012".to_string(),
///     arn: "arn:aws:iam::123456789012:user/alice".to_string(),
///     wami_arn: "arn:wami:iam:root:wami:123456789012:user/alice".parse().unwrap(),
///     providers: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerIdentity {
    /// The unique identifier of the calling entity
    pub user_id: String,
    /// The AWS account ID
    pub account: String,
    /// The ARN of the calling entity
    pub arn: String,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: WamiArn,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
