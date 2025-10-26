//! MfaDevice Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an MFA (Multi-Factor Authentication) device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaDevice {
    /// The user with whom the MFA device is associated
    pub user_name: String,
    /// The serial number that uniquely identifies the MFA device
    pub serial_number: String,
    /// The date when the MFA device was enabled
    pub enable_date: DateTime<Utc>,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
