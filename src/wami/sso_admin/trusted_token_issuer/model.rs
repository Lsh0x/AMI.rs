//! Trusted Token Issuer Model

use serde::{Deserialize, Serialize};

/// Represents a trusted token issuer for federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedTokenIssuer {
    /// The ARN of the trusted token issuer
    pub trusted_token_issuer_arn: String,
    /// The instance ARN this issuer belongs to
    pub instance_arn: String,
    /// The name of the trusted token issuer
    pub name: String,
    /// The type of the trusted token issuer (e.g., "OIDC_JWT")
    pub trusted_token_issuer_type: String,
    /// The issuer URL
    pub issuer_url: String,
    /// The WAMI ARN for cross-provider identification
    pub wami_arn: String,
    /// List of cloud providers where this resource exists
    pub providers: Vec<crate::provider::ProviderConfig>,
}
