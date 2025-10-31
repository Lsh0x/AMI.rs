//! Trusted Token Issuer Service
//!
//! Orchestrates trusted token issuer operations.

use crate::error::Result;
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::TrustedTokenIssuerStore;
use crate::wami::sso_admin::trusted_token_issuer::TrustedTokenIssuer;
use std::sync::{Arc, RwLock};

/// Service for managing trusted token issuers
pub struct TrustedTokenIssuerService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
}

impl<S: TrustedTokenIssuerStore> TrustedTokenIssuerService<S> {
    /// Create a new TrustedTokenIssuerService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
        }
    }

    /// Create a new trusted token issuer
    pub async fn create_trusted_token_issuer(
        &self,
        issuer: TrustedTokenIssuer,
    ) -> Result<TrustedTokenIssuer> {
        self.store
            .write()
            .unwrap()
            .create_trusted_token_issuer(issuer)
            .await
    }

    /// Get a trusted token issuer by ARN
    pub async fn get_trusted_token_issuer(
        &self,
        issuer_arn: &str,
    ) -> Result<Option<TrustedTokenIssuer>> {
        self.store
            .read()
            .unwrap()
            .get_trusted_token_issuer(issuer_arn)
            .await
    }

    /// Delete a trusted token issuer
    pub async fn delete_trusted_token_issuer(&self, issuer_arn: &str) -> Result<()> {
        self.store
            .write()
            .unwrap()
            .delete_trusted_token_issuer(issuer_arn)
            .await
    }

    /// List trusted token issuers for an instance
    pub async fn list_trusted_token_issuers(
        &self,
        instance_arn: &str,
    ) -> Result<Vec<TrustedTokenIssuer>> {
        self.store
            .read()
            .unwrap()
            .list_trusted_token_issuers(instance_arn)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::memory::InMemoryWamiStore;
    use chrono::Utc;

    fn setup_service() -> TrustedTokenIssuerService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        TrustedTokenIssuerService::new(store)
    }

    fn create_test_issuer(name: &str, instance_arn: &str) -> TrustedTokenIssuer {
        TrustedTokenIssuer {
            issuer_arn: format!("arn:aws:sso:::issuer/{}/tti-{}", instance_arn, name),
            name: Some(name.to_string()),
            issuer_url: format!("https://{}.example.com", name),
            instance_arn: instance_arn.to_string(),
            trusted_token_issuer_type: "OIDC".to_string(),
            created_date: Utc::now(),
            wami_arn: format!(
                "arn:wami:sso-admin:root:wami:123456789012:trusted-token-issuer/tti-{}",
                name
            )
            .parse()
            .unwrap(),
            providers: vec![],
        }
    }

    #[tokio::test]
    async fn test_create_and_get_issuer() {
        let service = setup_service();
        let issuer = create_test_issuer("okta", "instance-1");

        let created = service
            .create_trusted_token_issuer(issuer.clone())
            .await
            .unwrap();
        assert_eq!(created.name, Some("okta".to_string()));

        let retrieved = service
            .get_trusted_token_issuer(&issuer.issuer_arn)
            .await
            .unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_delete_issuer() {
        let service = setup_service();
        let issuer = create_test_issuer("temp", "instance-1");

        service
            .create_trusted_token_issuer(issuer.clone())
            .await
            .unwrap();
        service
            .delete_trusted_token_issuer(&issuer.issuer_arn)
            .await
            .unwrap();

        let retrieved = service
            .get_trusted_token_issuer(&issuer.issuer_arn)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_issuers() {
        let service = setup_service();
        let instance_arn = "instance-1";

        service
            .create_trusted_token_issuer(create_test_issuer("issuer1", instance_arn))
            .await
            .unwrap();
        service
            .create_trusted_token_issuer(create_test_issuer("issuer2", instance_arn))
            .await
            .unwrap();

        let issuers = service
            .list_trusted_token_issuers(instance_arn)
            .await
            .unwrap();
        assert_eq!(issuers.len(), 2);
    }
}
