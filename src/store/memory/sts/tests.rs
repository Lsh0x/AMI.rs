//! Tests for STS Store Implementations
//!
//! Tests for SessionStore and IdentityStore

use crate::provider::ProviderConfig;
use crate::store::memory::sts::InMemoryStsStore;
use crate::store::traits::{IdentityStore, SessionStore};
use crate::wami::sts::identity::CallerIdentity;
use crate::wami::sts::session::{SessionStatus, StsSession};
use chrono::{Duration, Utc};

// ============================================================================
// SESSION STORE TESTS
// ============================================================================

#[tokio::test]
async fn test_session_create_and_get() {
    let mut store = InMemoryStsStore::default();

    let session = StsSession {
        session_token: "session-token-123".to_string(),
        access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
        secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
        expiration: Utc::now() + Duration::hours(1),
        status: SessionStatus::Active,
        assumed_role_arn: None,
        federated_user_name: None,
        principal_arn: Some("arn:aws:iam::123456789012:user/alice".to_string()),
        arn: "arn:wami:sts::session-token-123".to_string(),
        wami_arn: "arn:wami:sts::session/session-token-123".to_string(),
        providers: Vec::new(),
        tenant_id: None,
        created_at: Utc::now(),
        last_used: None,
    };

    // Create session
    let created = store.create_session(session.clone()).await.unwrap();
    assert_eq!(created.session_token, "session-token-123");

    // Get session
    let retrieved = store.get_session("session-token-123").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().session_token, "session-token-123");
}

#[tokio::test]
async fn test_session_get_nonexistent() {
    let store = InMemoryStsStore::default();

    let result = store.get_session("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_session_delete() {
    let mut store = InMemoryStsStore::default();

    let session = StsSession {
        session_token: "temp-session".to_string(),
        access_key_id: "AKIATEST".to_string(),
        secret_access_key: "secret".to_string(),
        expiration: Utc::now() + Duration::hours(1),
        status: SessionStatus::Active,
        assumed_role_arn: None,
        federated_user_name: None,
        principal_arn: Some("arn:aws:iam::123:user/bob".to_string()),
        arn: "arn:wami:sts::temp-session".to_string(),
        wami_arn: "arn:wami:sts::session/temp-session".to_string(),
        providers: Vec::new(),
        tenant_id: None,
        created_at: Utc::now(),
        last_used: None,
    };

    store.create_session(session).await.unwrap();

    // Delete session
    store.delete_session("temp-session").await.unwrap();

    // Verify deleted
    let result = store.get_session("temp-session").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_session_list_empty() {
    let store = InMemoryStsStore::default();

    let sessions = store.list_sessions(None).await.unwrap();
    assert_eq!(sessions.len(), 0);
}

#[tokio::test]
async fn test_session_list_multiple() {
    let mut store = InMemoryStsStore::default();

    // Create multiple sessions
    for i in 0..3 {
        let session = StsSession {
            session_token: format!("session-{}", i),
            access_key_id: format!("AKIA{}", i),
            secret_access_key: "secret".to_string(),
            expiration: Utc::now() + Duration::hours(1),
            status: SessionStatus::Active,
            assumed_role_arn: None,
            federated_user_name: None,
            principal_arn: Some(format!("arn:aws:iam::123:user/user{}", i)),
            arn: format!("arn:wami:sts::session-{}", i),
            wami_arn: format!("arn:wami:sts::session/session-{}", i),
            providers: Vec::new(),
            tenant_id: None,
            created_at: Utc::now(),
            last_used: None,
        };
        store.create_session(session).await.unwrap();
    }

    let sessions = store.list_sessions(None).await.unwrap();
    assert_eq!(sessions.len(), 3);
}

#[tokio::test]
async fn test_session_with_role() {
    let mut store = InMemoryStsStore::default();

    let session = StsSession {
        session_token: "role-session".to_string(),
        access_key_id: "AKIAROLE".to_string(),
        secret_access_key: "rolesecret".to_string(),
        expiration: Utc::now() + Duration::hours(1),
        status: SessionStatus::Active,
        assumed_role_arn: Some("arn:aws:iam::123:role/AdminRole".to_string()),
        federated_user_name: None,
        principal_arn: Some("arn:aws:iam::123:user/alice".to_string()),
        arn: "arn:wami:sts::role-session".to_string(),
        wami_arn: "arn:wami:sts::session/role-session".to_string(),
        providers: Vec::new(),
        tenant_id: None,
        created_at: Utc::now(),
        last_used: None,
    };

    store.create_session(session.clone()).await.unwrap();

    let retrieved = store.get_session("role-session").await.unwrap().unwrap();
    assert!(retrieved.assumed_role_arn.is_some());
    assert_eq!(
        retrieved.assumed_role_arn.unwrap(),
        "arn:aws:iam::123:role/AdminRole"
    );
}

#[tokio::test]
async fn test_session_with_providers() {
    let mut store = InMemoryStsStore::default();

    let provider_config = ProviderConfig {
        provider_name: "aws".to_string(),
        account_id: "123456789012".to_string(),
        native_arn: "arn:aws:sts::123456789012:session/test".to_string(),
        synced_at: Utc::now(),
        tenant_id: None,
    };

    let session = StsSession {
        session_token: "multi-provider-session".to_string(),
        access_key_id: "AKIAMP".to_string(),
        secret_access_key: "secret".to_string(),
        expiration: Utc::now() + Duration::hours(1),
        status: SessionStatus::Active,
        assumed_role_arn: None,
        federated_user_name: None,
        principal_arn: Some("arn:aws:iam::123:user/test".to_string()),
        arn: "arn:wami:sts::multi-provider-session".to_string(),
        wami_arn: "arn:wami:sts::session/multi-provider-session".to_string(),
        providers: vec![provider_config],
        tenant_id: None,
        created_at: Utc::now(),
        last_used: None,
    };

    store.create_session(session).await.unwrap();

    let retrieved = store
        .get_session("multi-provider-session")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.providers.len(), 1);
    assert_eq!(retrieved.providers[0].provider_name, "aws");
}

// ============================================================================
// IDENTITY STORE TESTS
// ============================================================================

#[tokio::test]
async fn test_identity_create_and_get() {
    let mut store = InMemoryStsStore::default();

    let identity = CallerIdentity {
        user_id: "AIDACKCEVSQ6C2EXAMPLE".to_string(),
        account: "123456789012".to_string(),
        arn: "arn:aws:iam::123456789012:user/alice".to_string(),
        wami_arn: "arn:wami:iam::hash123:user/alice".to_string(),
        providers: Vec::new(),
    };

    let arn = identity.arn.clone();

    // Create identity
    let created = store.create_identity(identity.clone()).await.unwrap();
    assert_eq!(created.user_id, "AIDACKCEVSQ6C2EXAMPLE");

    // Get identity
    let retrieved = store.get_identity(&arn).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().user_id, "AIDACKCEVSQ6C2EXAMPLE");
}

#[tokio::test]
async fn test_identity_get_nonexistent() {
    let store = InMemoryStsStore::default();

    let result = store
        .get_identity("arn:aws:iam::123:user/nonexistent")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_identity_list_empty() {
    let store = InMemoryStsStore::default();

    let identities = store.list_identities().await.unwrap();
    assert_eq!(identities.len(), 0);
}

#[tokio::test]
async fn test_identity_list_multiple() {
    let mut store = InMemoryStsStore::default();

    // Create multiple identities
    for i in 0..3 {
        let identity = CallerIdentity {
            user_id: format!("USERID{}", i),
            account: "123456789012".to_string(),
            arn: format!("arn:aws:iam::123456789012:user/user{}", i),
            wami_arn: format!("arn:wami:iam::hash:user/user{}", i),
            providers: Vec::new(),
        };
        store.create_identity(identity).await.unwrap();
    }

    let identities = store.list_identities().await.unwrap();
    assert_eq!(identities.len(), 3);
}

#[tokio::test]
async fn test_identity_with_providers() {
    let mut store = InMemoryStsStore::default();

    let provider_config = ProviderConfig {
        provider_name: "aws".to_string(),
        account_id: "123456789012".to_string(),
        native_arn: "arn:aws:iam::123456789012:user/alice".to_string(),
        synced_at: Utc::now(),
        tenant_id: None,
    };

    let identity = CallerIdentity {
        user_id: "AIDATEST".to_string(),
        account: "123456789012".to_string(),
        arn: "arn:aws:iam::123456789012:user/alice".to_string(),
        wami_arn: "arn:wami:iam::hash:user/alice".to_string(),
        providers: vec![provider_config],
    };

    store.create_identity(identity.clone()).await.unwrap();

    let retrieved = store.get_identity(&identity.arn).await.unwrap().unwrap();
    assert_eq!(retrieved.providers.len(), 1);
    assert_eq!(retrieved.providers[0].provider_name, "aws");
}

#[tokio::test]
async fn test_identity_update() {
    let mut store = InMemoryStsStore::default();

    let identity = CallerIdentity {
        user_id: "USER1".to_string(),
        account: "111111111111".to_string(),
        arn: "arn:aws:iam::111111111111:user/test".to_string(),
        wami_arn: "arn:wami:iam::hash:user/test".to_string(),
        providers: Vec::new(),
    };

    store.create_identity(identity.clone()).await.unwrap();

    // Update (create with same ARN replaces)
    let updated_identity = CallerIdentity {
        user_id: "USER1".to_string(),
        account: "222222222222".to_string(), // Changed account
        arn: "arn:aws:iam::111111111111:user/test".to_string(),
        wami_arn: "arn:wami:iam::hash:user/test".to_string(),
        providers: Vec::new(),
    };

    store.create_identity(updated_identity).await.unwrap();

    let retrieved = store.get_identity(&identity.arn).await.unwrap().unwrap();
    assert_eq!(retrieved.account, "222222222222");
}
