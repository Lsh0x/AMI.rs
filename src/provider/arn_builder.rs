//! WAMI ARN Builder
//!
//! Provides secure, opaque ARN generation for cross-cloud resource identification.
//!
//! # Security Model
//!
//! WAMI ARNs use opaque tenant identifiers to prevent information leakage:
//! - Real account ID: `123456789012`
//! - WAMI ARN: `arn:wami:iam:tenant-a1b2c3d4:user/alice`
//!
//! The tenant hash is deterministic (same account → same hash) but irreversible
//! without database access.
//!
//! # Example
//!
//! ```rust
//! use wami::provider::arn_builder::WamiArnBuilder;
//!
//! let builder = WamiArnBuilder::new();
//! let arn = builder.build_arn(
//!     "iam",
//!     "123456789012",
//!     "user",
//!     "/tenants/acme/",
//!     "alice"
//! );
//!
//! // Result: arn:wami:iam:tenant-a1b2c3d4:user/tenants/acme/alice
//! assert!(arn.starts_with("arn:wami:iam:tenant-"));
//! ```

use crate::error::{AmiError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// WAMI ARN Builder for generating secure, opaque ARNs
#[derive(Debug, Clone)]
pub struct WamiArnBuilder {
    /// Optional salt for hashing account IDs
    /// Can be set via WAMI_ARN_SALT environment variable
    salt: Option<String>,
}

impl WamiArnBuilder {
    /// Creates a new ARN builder
    ///
    /// Automatically reads salt from `WAMI_ARN_SALT` environment variable if available.
    pub fn new() -> Self {
        Self {
            salt: std::env::var("WAMI_ARN_SALT").ok(),
        }
    }

    /// Creates a builder with explicit salt
    pub fn with_salt(salt: impl Into<String>) -> Self {
        Self {
            salt: Some(salt.into()),
        }
    }

    /// Builds a WAMI ARN
    ///
    /// # Arguments
    ///
    /// * `service` - Service name ("iam", "sts", "tenant")
    /// * `account_id` - Real account/project ID
    /// * `resource_type` - Resource type ("user", "role", "policy", etc.)
    /// * `path` - Resource path (e.g., "/tenants/acme/")
    /// * `name` - Resource name
    ///
    /// # Returns
    ///
    /// A WAMI ARN in format: `arn:wami:<service>:tenant-<hash>:<resource_type><path><name>`
    pub fn build_arn(
        &self,
        service: &str,
        account_id: &str,
        resource_type: &str,
        path: &str,
        name: &str,
    ) -> String {
        let tenant_hash = self.hash_account(account_id);
        format!(
            "arn:wami:{}:{}:{}{}{}",
            service, tenant_hash, resource_type, path, name
        )
    }

    /// Hashes an account ID to create an opaque tenant identifier
    ///
    /// Uses SHA-256 with optional salt for security.
    /// Returns format: `tenant-<8-hex-chars>`
    fn hash_account(&self, account_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(account_id.as_bytes());

        if let Some(salt) = &self.salt {
            hasher.update(salt.as_bytes());
        }

        let result = hasher.finalize();
        // Take first 4 bytes (8 hex chars) for compact representation
        format!("tenant-{}", hex::encode(&result[..4]))
    }

    /// Gets the configured salt (if any)
    pub fn salt(&self) -> Option<&str> {
        self.salt.as_deref()
    }
}

impl Default for WamiArnBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed WAMI ARN components
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedArn {
    /// Provider (always "wami")
    pub provider: String,
    /// Service (iam, sts, tenant)
    pub service: String,
    /// Opaque tenant hash (tenant-xxxxxxxx)
    pub tenant_hash: String,
    /// Resource type (user, role, policy, etc.)
    pub resource_type: String,
    /// Resource path
    pub path: String,
    /// Resource name
    pub name: String,
}

impl ParsedArn {
    /// Parses a WAMI ARN string
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::provider::arn_builder::ParsedArn;
    ///
    /// let parsed = ParsedArn::from_arn("arn:wami:iam:tenant-abc123:user/alice").unwrap();
    /// assert_eq!(parsed.service, "iam");
    /// assert_eq!(parsed.resource_type, "user");
    /// assert_eq!(parsed.name, "alice");
    /// ```
    #[allow(clippy::result_large_err)]
    pub fn from_arn(arn: &str) -> Result<Self> {
        let parts: Vec<&str> = arn.split(':').collect();

        if parts.len() < 5 {
            return Err(AmiError::InvalidParameter {
                message: format!("Invalid ARN format: {}", arn),
            });
        }

        if parts[0] != "arn" {
            return Err(AmiError::InvalidParameter {
                message: format!("ARN must start with 'arn:', got: {}", arn),
            });
        }

        if parts[1] != "wami" {
            return Err(AmiError::InvalidParameter {
                message: format!("Expected 'wami' provider, got: {}", parts[1]),
            });
        }

        let provider = parts[1].to_string();
        let service = parts[2].to_string();
        let tenant_hash = parts[3].to_string();
        let resource_path = parts[4];

        // Parse resource_type/path/name
        let (resource_type, path, name) = Self::parse_resource_path(resource_path)?;

        Ok(ParsedArn {
            provider,
            service,
            tenant_hash,
            resource_type,
            path,
            name,
        })
    }

    /// Parses the resource path component
    ///
    /// Examples:
    /// - `user/alice` → ("user", "", "alice")
    /// - `user/tenants/acme/alice` → ("user", "/tenants/acme/", "alice")
    /// - `role/Admin` → ("role", "", "Admin")
    #[allow(clippy::result_large_err)]
    fn parse_resource_path(resource_path: &str) -> Result<(String, String, String)> {
        let parts: Vec<&str> = resource_path.split('/').collect();

        if parts.is_empty() {
            return Err(AmiError::InvalidParameter {
                message: "Empty resource path".to_string(),
            });
        }

        let resource_type = parts[0].to_string();

        if parts.len() == 1 {
            return Err(AmiError::InvalidParameter {
                message: format!("Missing resource name in: {}", resource_path),
            });
        }

        if parts.len() == 2 {
            // Simple case: type/name
            return Ok((resource_type, String::new(), parts[1].to_string()));
        }

        // Complex case: type/path/name
        let name = parts[parts.len() - 1].to_string();
        let path_parts = &parts[1..parts.len() - 1];
        let path = format!("/{}/", path_parts.join("/"));

        Ok((resource_type, path, name))
    }

    /// Reconstructs the full ARN
    pub fn to_arn(&self) -> String {
        if self.path.is_empty() {
            format!(
                "arn:{}:{}:{}:{}/{}",
                self.provider, self.service, self.tenant_hash, self.resource_type, self.name
            )
        } else {
            format!(
                "arn:{}:{}:{}:{}{}{}",
                self.provider,
                self.service,
                self.tenant_hash,
                self.resource_type,
                self.path,
                self.name
            )
        }
    }

    /// Checks if this ARN matches a wildcard pattern
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::provider::arn_builder::ParsedArn;
    ///
    /// let arn = ParsedArn::from_arn("arn:wami:iam:tenant-abc:user/alice").unwrap();
    /// assert!(arn.matches_pattern("arn:wami:iam:tenant-abc:user/*"));
    /// assert!(arn.matches_pattern("arn:wami:iam:tenant-abc:*"));
    /// assert!(!arn.matches_pattern("arn:wami:sts:*:*"));
    /// ```
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        arn_pattern_match(&self.to_arn(), pattern)
    }
}

impl fmt::Display for ParsedArn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_arn())
    }
}

/// Matches an ARN against a wildcard pattern
///
/// Supports `*` (match any) and `?` (match single char) wildcards.
pub fn arn_pattern_match(arn: &str, pattern: &str) -> bool {
    // Convert IAM-style wildcards to regex
    let escaped = regex::escape(pattern);
    let with_wildcards = escaped.replace(r"\*", ".*").replace(r"\?", ".");
    let regex_pattern = format!("^{}$", with_wildcards);

    if let Ok(re) = regex::Regex::new(&regex_pattern) {
        re.is_match(arn)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arn_builder_basic() {
        let builder = WamiArnBuilder::new();
        let arn = builder.build_arn("iam", "123456789012", "user", "/", "alice");

        assert!(arn.starts_with("arn:wami:iam:tenant-"));
        assert!(arn.ends_with(":user/alice"));
    }

    #[test]
    fn test_arn_builder_with_path() {
        let builder = WamiArnBuilder::new();
        let arn = builder.build_arn("iam", "123456789012", "user", "/tenants/acme/", "alice");

        assert!(arn.contains("user/tenants/acme/alice"));
    }

    #[test]
    fn test_arn_builder_deterministic() {
        let builder = WamiArnBuilder::with_salt("test-salt");
        let arn1 = builder.build_arn("iam", "123456789012", "user", "/", "alice");
        let arn2 = builder.build_arn("iam", "123456789012", "user", "/", "bob");

        // Same account → same tenant hash
        let hash1 = arn1.split(':').nth(3).unwrap();
        let hash2 = arn2.split(':').nth(3).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_arn_builder_different_accounts() {
        let builder = WamiArnBuilder::new();
        let arn1 = builder.build_arn("iam", "123456789012", "user", "/", "alice");
        let arn2 = builder.build_arn("iam", "999999999999", "user", "/", "alice");

        // Different accounts → different hashes
        let hash1 = arn1.split(':').nth(3).unwrap();
        let hash2 = arn2.split(':').nth(3).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_parse_arn_simple() {
        let parsed = ParsedArn::from_arn("arn:wami:iam:tenant-abc123:user/alice").unwrap();

        assert_eq!(parsed.provider, "wami");
        assert_eq!(parsed.service, "iam");
        assert_eq!(parsed.tenant_hash, "tenant-abc123");
        assert_eq!(parsed.resource_type, "user");
        assert_eq!(parsed.path, "");
        assert_eq!(parsed.name, "alice");
    }

    #[test]
    fn test_parse_arn_with_path() {
        let parsed =
            ParsedArn::from_arn("arn:wami:iam:tenant-abc:user/tenants/acme/engineering/alice")
                .unwrap();

        assert_eq!(parsed.resource_type, "user");
        assert_eq!(parsed.path, "/tenants/acme/engineering/");
        assert_eq!(parsed.name, "alice");
    }

    #[test]
    fn test_parse_arn_roundtrip() {
        let original = "arn:wami:iam:tenant-abc:user/tenants/acme/alice";
        let parsed = ParsedArn::from_arn(original).unwrap();
        let reconstructed = parsed.to_arn();

        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_arn_pattern_match_exact() {
        assert!(arn_pattern_match(
            "arn:wami:iam:tenant-abc:user/alice",
            "arn:wami:iam:tenant-abc:user/alice"
        ));
    }

    #[test]
    fn test_arn_pattern_match_wildcard() {
        assert!(arn_pattern_match(
            "arn:wami:iam:tenant-abc:user/alice",
            "arn:wami:iam:tenant-abc:user/*"
        ));

        assert!(arn_pattern_match(
            "arn:wami:iam:tenant-abc:user/alice",
            "arn:wami:iam:tenant-abc:*"
        ));

        assert!(arn_pattern_match(
            "arn:wami:iam:tenant-abc:user/alice",
            "arn:wami:*:*:*"
        ));
    }

    #[test]
    fn test_arn_pattern_no_match() {
        assert!(!arn_pattern_match(
            "arn:wami:iam:tenant-abc:user/alice",
            "arn:wami:sts:*:*"
        ));

        assert!(!arn_pattern_match(
            "arn:wami:iam:tenant-abc:user/alice",
            "arn:wami:iam:tenant-xyz:*"
        ));
    }

    #[test]
    fn test_parsed_arn_matches_pattern() {
        let arn = ParsedArn::from_arn("arn:wami:iam:tenant-abc:user/tenants/acme/alice").unwrap();

        assert!(arn.matches_pattern("arn:wami:iam:tenant-abc:user/*"));
        assert!(arn.matches_pattern("arn:wami:iam:tenant-abc:user/tenants/acme/*"));
        assert!(!arn.matches_pattern("arn:wami:sts:*:*"));
    }
}
