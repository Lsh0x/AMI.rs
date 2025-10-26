//! STS Store Trait
//!
//! Defines the interface for STS (Security Token Service) data storage operations.

use crate::error::Result;
use crate::sts::{CallerIdentity, StsSession};
use async_trait::async_trait;

/// Trait for STS data storage operations
#[async_trait]
pub trait StsStore: Send + Sync {
    /// Get the account ID for this store
    fn account_id(&self) -> &str;

    // Session operations
    async fn create_session(&mut self, session: StsSession) -> Result<StsSession>;
    async fn get_session(&self, session_token: &str) -> Result<Option<StsSession>>;
    async fn delete_session(&mut self, session_token: &str) -> Result<()>;
    async fn list_sessions(&self, user_id: Option<&str>) -> Result<Vec<StsSession>>;

    // Identity operations
    async fn create_identity(&mut self, identity: CallerIdentity) -> Result<CallerIdentity>;
    async fn get_identity(&self, arn: &str) -> Result<Option<CallerIdentity>>;
    async fn list_identities(&self) -> Result<Vec<CallerIdentity>>;
}
