//! AccessKey Domain Model

use crate::arn::WamiArn;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an IAM access key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKey {
    /// The name of the IAM user that the key is associated with
    pub user_name: String,
    /// The ID for this access key
    pub access_key_id: String,
    /// The status of the access key: Active or Inactive
    pub status: String,
    /// The date when the access key was created
    pub create_date: DateTime<Utc>,
    /// The secret key used to sign requests (only provided when creating the key)
    pub secret_access_key: Option<String>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: WamiArn,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}

/// Represents the last time an access key was used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKeyLastUsed {
    /// The date and time when the access key was last used
    pub last_used_date: Option<DateTime<Utc>>,
    /// The AWS region where the access key was last used
    pub region: Option<String>,
    /// The AWS service that was accessed
    pub service_name: Option<String>,
}
