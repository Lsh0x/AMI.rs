//! Identity Provider Store Trait
//!
//! Focused trait for SAML and OIDC identity provider storage operations.

use crate::error::Result;
use crate::types::{PaginationParams, Tag};
use crate::wami::identity::identity_provider::{OidcProvider, SamlProvider};
use async_trait::async_trait;

/// Store trait for Identity Provider operations
#[async_trait]
pub trait IdentityProviderStore: Send + Sync {
    // ===========================
    // SAML Provider Operations
    // ===========================

    /// Create a new SAML provider
    async fn create_saml_provider(&mut self, provider: SamlProvider) -> Result<SamlProvider>;

    /// Get a SAML provider by ARN
    async fn get_saml_provider(&self, arn: &str) -> Result<Option<SamlProvider>>;

    /// Update an existing SAML provider
    async fn update_saml_provider(&mut self, provider: SamlProvider) -> Result<SamlProvider>;

    /// Delete a SAML provider
    async fn delete_saml_provider(&mut self, arn: &str) -> Result<()>;

    /// List SAML providers with optional pagination
    async fn list_saml_providers(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<SamlProvider>, bool, Option<String>)>;

    // ===========================
    // OIDC Provider Operations
    // ===========================

    /// Create a new OIDC provider
    async fn create_oidc_provider(&mut self, provider: OidcProvider) -> Result<OidcProvider>;

    /// Get an OIDC provider by ARN
    async fn get_oidc_provider(&self, arn: &str) -> Result<Option<OidcProvider>>;

    /// Update an existing OIDC provider
    async fn update_oidc_provider(&mut self, provider: OidcProvider) -> Result<OidcProvider>;

    /// Delete an OIDC provider
    async fn delete_oidc_provider(&mut self, arn: &str) -> Result<()>;

    /// List OIDC providers with optional pagination
    async fn list_oidc_providers(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<OidcProvider>, bool, Option<String>)>;

    // ===========================
    // Tagging Operations
    // ===========================

    /// Tag an identity provider (SAML or OIDC)
    async fn tag_identity_provider(&mut self, arn: &str, tags: Vec<Tag>) -> Result<()>;

    /// List tags for an identity provider
    async fn list_identity_provider_tags(&self, arn: &str) -> Result<Vec<Tag>>;

    /// Untag an identity provider
    async fn untag_identity_provider(&mut self, arn: &str, tag_keys: Vec<String>) -> Result<()>;
}
