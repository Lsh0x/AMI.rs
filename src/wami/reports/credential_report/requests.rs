//! Report Request and Response Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::model::*;

/// Request to get the credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCredentialReportRequest {}

/// Response from getting the credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCredentialReportResponse {
    /// The credential report in CSV format (base64 encoded)
    #[serde(rename = "Content")]
    pub content: String,

    /// The format of the report (always "text/csv")
    #[serde(rename = "ReportFormat")]
    pub report_format: String,

    /// When the report was generated
    #[serde(rename = "GeneratedTime")]
    pub generated_time: DateTime<Utc>,
}

/// Request to generate a new credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCredentialReportRequest {}

/// Response from generating a credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCredentialReportResponse {
    /// The state of the report generation
    #[serde(rename = "State")]
    pub state: ReportState,

    /// Description of the report state
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request to get account summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountSummaryRequest {}

/// Response from getting account summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountSummaryResponse {
    /// Summary map of resource counts and limits
    #[serde(rename = "SummaryMap")]
    pub summary_map: AccountSummaryMap,
}
