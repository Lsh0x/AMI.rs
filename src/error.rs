use thiserror::Error;

#[derive(Error, Debug)]
pub enum AmiError {
    #[error("AWS SDK error: {0}")]
    AwsSdk(#[from] aws_sdk_iam::Error),

    #[error("STS SDK error: {0}")]
    StsSdk(#[from] aws_sdk_sts::Error),

    #[error("SSO Admin SDK error: {0}")]
    SsoAdminSdk(#[from] aws_sdk_ssoadmin::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    #[error("Operation not supported: {operation}")]
    OperationNotSupported { operation: String },

    #[error("Resource not found: {resource}")]
    ResourceNotFound { resource: String },

    #[error("Permission denied: {reason}")]
    PermissionDenied { reason: String },

    #[error("Access denied: {message}")]
    AccessDenied { message: String },

    #[error("Resource limit exceeded: {resource_type} limit is {limit}")]
    ResourceLimitExceeded { resource_type: String, limit: usize },

    #[error("Resource already exists: {resource}")]
    ResourceExists { resource: String },

    #[error("Store error: {0}")]
    StoreError(String),
}

pub type Result<T> = std::result::Result<T, AmiError>;
