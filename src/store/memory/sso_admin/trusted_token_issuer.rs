//! Trusted Token Issuer Store Implementation for InMemorySsoAdminStore

use crate::error::Result;
use crate::store::memory::sso_admin::InMemorySsoAdminStore;
use crate::store::traits::TrustedTokenIssuerStore;
use crate::wami::sso_admin::TrustedTokenIssuer;
use async_trait::async_trait;

#[async_trait]
impl TrustedTokenIssuerStore for InMemorySsoAdminStore {
    async fn create_trusted_token_issuer(
        &mut self,
        issuer: TrustedTokenIssuer,
    ) -> Result<TrustedTokenIssuer> {
        self.trusted_token_issuers
            .insert(issuer.issuer_arn.clone(), issuer.clone());
        Ok(issuer)
    }

    async fn get_trusted_token_issuer(
        &self,
        issuer_arn: &str,
    ) -> Result<Option<TrustedTokenIssuer>> {
        Ok(self.trusted_token_issuers.get(issuer_arn).cloned())
    }

    async fn delete_trusted_token_issuer(&mut self, issuer_arn: &str) -> Result<()> {
        self.trusted_token_issuers.remove(issuer_arn);
        Ok(())
    }

    async fn list_trusted_token_issuers(
        &self,
        _instance_arn: &str,
    ) -> Result<Vec<TrustedTokenIssuer>> {
        Ok(self.trusted_token_issuers.values().cloned().collect())
    }
}

/// Implement TrustedTokenIssuerStore for InMemoryWamiStore (the main unified store)
#[async_trait]
impl TrustedTokenIssuerStore for super::super::wami::InMemoryWamiStore {
    async fn create_trusted_token_issuer(
        &mut self,
        issuer: TrustedTokenIssuer,
    ) -> Result<TrustedTokenIssuer> {
        self.trusted_token_issuers
            .insert(issuer.issuer_arn.clone(), issuer.clone());
        Ok(issuer)
    }

    async fn get_trusted_token_issuer(
        &self,
        issuer_arn: &str,
    ) -> Result<Option<TrustedTokenIssuer>> {
        Ok(self.trusted_token_issuers.get(issuer_arn).cloned())
    }

    async fn delete_trusted_token_issuer(&mut self, issuer_arn: &str) -> Result<()> {
        self.trusted_token_issuers.remove(issuer_arn);
        Ok(())
    }

    async fn list_trusted_token_issuers(
        &self,
        instance_arn: &str,
    ) -> Result<Vec<TrustedTokenIssuer>> {
        Ok(self
            .trusted_token_issuers
            .values()
            .filter(|tti| tti.instance_arn == instance_arn)
            .cloned()
            .collect())
    }
}
