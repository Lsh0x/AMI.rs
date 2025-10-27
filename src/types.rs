use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Common response wrapper for AWS operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> AmiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// AWS region configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub region: String,
    pub profile: Option<String>,
    pub account_id: String,
}

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            profile: None,
            account_id: Self::generate_account_id(),
        }
    }
}

impl AwsConfig {
    /// Generate a random AWS account ID (12 digits)
    pub fn generate_account_id() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or(0)
            .hash(&mut hasher);
        let hash = hasher.finish();

        // Generate 12-digit account ID
        format!("{:012}", hash % 1_000_000_000_000)
    }

    /// Create a new config with a specific account ID
    pub fn with_account_id(account_id: String) -> Self {
        Self {
            region: "us-east-1".to_string(),
            profile: None,
            account_id,
        }
    }
}

/// Common pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub max_items: Option<i32>,
    pub marker: Option<String>,
}

/// Tag representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// Policy document representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDocument {
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Statement")]
    pub statement: Vec<PolicyStatement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatement {
    #[serde(rename = "Effect")]
    pub effect: String,
    #[serde(rename = "Action", deserialize_with = "string_or_vec")]
    pub action: Vec<String>,
    #[serde(rename = "Resource", deserialize_with = "string_or_vec")]
    pub resource: Vec<String>,
    #[serde(rename = "Condition", skip_serializing_if = "Option::is_none")]
    pub condition: Option<Value>,
}

/// Deserialize either a single string or an array of strings into a Vec<String>
fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(vec![s]),
        Value::Array(arr) => arr
            .into_iter()
            .map(|v| {
                v.as_str()
                    .map(String::from)
                    .ok_or_else(|| serde::de::Error::custom("expected string"))
            })
            .collect(),
        _ => Err(serde::de::Error::custom(
            "expected string or array of strings",
        )),
    }
}
