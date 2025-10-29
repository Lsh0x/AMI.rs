//! Session Store Trait
//!
//! Defines the interface for STS session storage operations.

use crate::error::Result;
use crate::wami::sts::StsSession;
use async_trait::async_trait;

/// Trait for STS session storage operations
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new STS session
    async fn create_session(&mut self, session: StsSession) -> Result<StsSession>;

    /// Retrieve a session by its session token
    async fn get_session(&self, session_token: &str) -> Result<Option<StsSession>>;

    /// Delete a session by its session token
    async fn delete_session(&mut self, session_token: &str) -> Result<()>;

    /// List all sessions, optionally filtered by user ID
    async fn list_sessions(&self, user_id: Option<&str>) -> Result<Vec<StsSession>>;
}
