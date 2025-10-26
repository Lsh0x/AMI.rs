//! Session Operations

use super::model::*;
use crate::error::{AmiError, Result};
use crate::store::{Store, StsStore};
use crate::sts::StsClient;
use crate::types::AmiResponse;

impl<S: Store> StsClient<S>
where
    S::StsStore: StsStore,
{
    /// Validate a session token
    pub async fn validate_session(&mut self, session_token: &str) -> Result<bool> {
        let store = self.sts_store().await?;

        if let Some(mut session) = store.get_session(session_token).await? {
            session.update_status();

            // Update session status in store if it changed
            if session.status != SessionStatus::Active {
                store.create_session(session.clone()).await?;
            }

            Ok(session.is_valid())
        } else {
            Ok(false)
        }
    }

    /// Revoke a session token
    pub async fn revoke_session(&mut self, session_token: &str) -> Result<AmiResponse<()>> {
        let store = self.sts_store().await?;

        let mut session =
            store
                .get_session(session_token)
                .await?
                .ok_or_else(|| AmiError::ResourceNotFound {
                    resource: format!("Session {} not found", session_token),
                })?;

        session.revoke();
        store.create_session(session).await?;

        Ok(AmiResponse::success(()))
    }

    /// Get session information
    pub async fn get_session(&mut self, session_token: &str) -> Result<AmiResponse<StsSession>> {
        let store = self.sts_store().await?;

        let mut session =
            store
                .get_session(session_token)
                .await?
                .ok_or_else(|| AmiError::ResourceNotFound {
                    resource: format!("Session {} not found", session_token),
                })?;

        session.update_status();
        session.touch();

        // Update last used time
        store.create_session(session.clone()).await?;

        Ok(AmiResponse::success(session))
    }

    /// List all sessions for a user
    pub async fn list_sessions(
        &mut self,
        user_id: Option<&str>,
    ) -> Result<AmiResponse<Vec<StsSession>>> {
        let store = self.sts_store().await?;
        let sessions = store.list_sessions(user_id).await?;

        Ok(AmiResponse::success(sessions))
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&mut self) -> Result<AmiResponse<usize>> {
        let store = self.sts_store().await?;
        let all_sessions = store.list_sessions(None).await?;
        let mut deleted = 0;

        for session in all_sessions {
            if session.is_expired() {
                store.delete_session(&session.session_token).await?;
                deleted += 1;
            }
        }

        Ok(AmiResponse::success(deleted))
    }
}
