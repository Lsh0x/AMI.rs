//! Identity Store Trait
//!
//! Defines the interface for caller identity storage operations.

use crate::error::Result;
use crate::wami::sts::CallerIdentity;
use async_trait::async_trait;

/// Trait for caller identity storage operations
#[async_trait]
pub trait IdentityStore: Send + Sync {
    /// Create a new caller identity
    async fn create_identity(&mut self, identity: CallerIdentity) -> Result<CallerIdentity>;

    /// Retrieve an identity by its ARN
    async fn get_identity(&self, arn: &str) -> Result<Option<CallerIdentity>>;

    /// List all identities
    async fn list_identities(&self) -> Result<Vec<CallerIdentity>>;
}
