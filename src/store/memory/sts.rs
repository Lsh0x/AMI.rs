//! In-memory STS Store Implementation

use crate::error::Result;
use crate::store::traits::StsStore;
use crate::sts::{CallerIdentity, StsSession};
use async_trait::async_trait;
use std::collections::HashMap;

/// In-memory implementation of STS store
///
/// This is a pure persistence layer that stores sessions and identities for ALL tenants.
/// Each session/identity carries its own tenant_id and account_id.
#[derive(Debug, Clone, Default)]
pub struct InMemoryStsStore {
    sessions: HashMap<String, StsSession>,
    identities: HashMap<String, CallerIdentity>,
}

impl InMemoryStsStore {
    /// Create a new empty STS store
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl StsStore for InMemoryStsStore {
    async fn create_session(&mut self, session: StsSession) -> Result<StsSession> {
        self.sessions
            .insert(session.session_token.clone(), session.clone());
        Ok(session)
    }

    async fn get_session(&self, session_token: &str) -> Result<Option<StsSession>> {
        Ok(self.sessions.get(session_token).cloned())
    }

    async fn delete_session(&mut self, session_token: &str) -> Result<()> {
        self.sessions.remove(session_token);
        Ok(())
    }

    async fn list_sessions(&self, _user_id: Option<&str>) -> Result<Vec<StsSession>> {
        let sessions: Vec<StsSession> = self.sessions.values().cloned().collect();
        Ok(sessions)
    }

    async fn create_identity(&mut self, identity: CallerIdentity) -> Result<CallerIdentity> {
        self.identities
            .insert(identity.arn.clone(), identity.clone());
        Ok(identity)
    }

    async fn get_identity(&self, arn: &str) -> Result<Option<CallerIdentity>> {
        Ok(self.identities.get(arn).cloned())
    }

    async fn list_identities(&self) -> Result<Vec<CallerIdentity>> {
        Ok(self.identities.values().cloned().collect())
    }
}
