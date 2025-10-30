//! STS Session Service
//!
//! Orchestrates STS session management operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::SessionStore;
use crate::wami::sts::StsSession;
use std::sync::{Arc, RwLock};

/// Service for managing STS sessions
///
/// Provides high-level operations for STS session CRUD.
pub struct SessionService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S: SessionStore> SessionService<S> {
    /// Create a new SessionService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>, account_id: String) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
            account_id,
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
            account_id: self.account_id.clone(),
        }
    }

    /// Create a new STS session
    pub async fn create_session(&self, session: StsSession) -> Result<StsSession> {
        self.store.write().unwrap().create_session(session).await
    }

    /// Get a session by session token
    pub async fn get_session(&self, session_token: &str) -> Result<Option<StsSession>> {
        self.store.read().unwrap().get_session(session_token).await
    }

    /// Delete a session
    pub async fn delete_session(&self, session_token: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_session(session_token)
            .await
    }

    /// List sessions, optionally filtered by user ID
    pub async fn list_sessions(&self, user_id: Option<&str>) -> Result<Vec<StsSession>> {
        self.store.read().unwrap().list_sessions(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::sts::StsSession;
    use chrono::{Duration, Utc};

    fn setup_service() -> SessionService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        SessionService::new(store, "123456789012".to_string())
    }

    fn create_test_session(session_name: &str) -> StsSession {
        StsSession {
            session_token: format!("token-{}", session_name),
            access_key_id: format!("AKIA{}", session_name),
            secret_access_key: "secret".to_string(),
            expiration: Utc::now() + Duration::hours(1),
            status: crate::wami::sts::session::SessionStatus::Active,
            assumed_role_arn: None,
            federated_user_name: None,
            principal_arn: Some(format!("arn:aws:iam::123456789012:user/{}", session_name)),
            arn: format!("arn:aws:sts::123456789012:session/{}", session_name),
            wami_arn: format!("arn:wami:sts::123456789012:session/{}", session_name),
            providers: vec![],
            tenant_id: None,
            created_at: Utc::now(),
            last_used: None,
        }
    }

    #[tokio::test]
    async fn test_create_and_get_session() {
        let service = setup_service();

        let session = create_test_session("alice");
        let created = service.create_session(session.clone()).await.unwrap();

        assert_eq!(created.session_token, "token-alice");

        let retrieved = service.get_session("token-alice").await.unwrap().unwrap();
        assert_eq!(retrieved.session_token, "token-alice");
    }

    #[tokio::test]
    async fn test_delete_session() {
        let service = setup_service();

        let session = create_test_session("bob");
        service.create_session(session).await.unwrap();

        service.delete_session("token-bob").await.unwrap();

        let retrieved = service.get_session("token-bob").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let service = setup_service();

        // Create sessions for different users
        for i in 0..3 {
            let session = create_test_session(&format!("user{}", i));
            service.create_session(session).await.unwrap();
        }

        let sessions = service.list_sessions(None).await.unwrap();
        assert_eq!(sessions.len(), 3);
    }
}
