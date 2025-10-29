//! Session Store Implementation for InMemoryStsStore

use crate::error::Result;
use crate::store::memory::sts::InMemoryStsStore;
use crate::store::traits::SessionStore;
use crate::wami::sts::StsSession;
use async_trait::async_trait;

#[async_trait]
impl SessionStore for InMemoryStsStore {
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
}
