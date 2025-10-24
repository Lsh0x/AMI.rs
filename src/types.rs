use serde::{Deserialize, Serialize};

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
}

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            profile: None,
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
    pub version: String,
    pub statement: Vec<PolicyStatement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatement {
    pub effect: String,
    pub action: Vec<String>,
    pub resource: Vec<String>,
    pub condition: Option<serde_json::Value>,
}
