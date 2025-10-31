//! Identity Provider Service
//!
//! Orchestrates identity provider management operations by combining wami builders with store persistence.

use crate::context::WamiContext;
use crate::error::{AmiError, Result};
use crate::store::traits::IdentityProviderStore;
use crate::types::Tag;
use crate::wami::identity::identity_provider::{
    builder, operations, AddClientIDToOpenIDConnectProviderRequest,
    CreateOpenIDConnectProviderRequest, CreateSAMLProviderRequest,
    ListOpenIDConnectProvidersRequest, ListSAMLProvidersRequest, OidcProvider,
    RemoveClientIDFromOpenIDConnectProviderRequest, SamlProvider,
    UpdateOpenIDConnectProviderThumbprintRequest, UpdateSAMLProviderRequest,
};
use std::sync::{Arc, RwLock};

/// Service for managing identity providers (SAML and OIDC)
///
/// Provides high-level operations for federated authentication setup.
pub struct IdentityProviderService<S> {
    store: Arc<RwLock<S>>,
}

impl<S: IdentityProviderStore> IdentityProviderService<S> {
    /// Create a new IdentityProviderService
    pub fn new(store: Arc<RwLock<S>>) -> Self {
        Self { store }
    }

    // ===========================
    // SAML Provider Operations
    // ===========================

    /// Create a new SAML provider
    pub async fn create_saml_provider(
        &self,
        context: &WamiContext,
        request: CreateSAMLProviderRequest,
    ) -> Result<SamlProvider> {
        // Validate name
        SamlProvider::validate_name(&request.name)?;

        // Validate metadata
        operations::validate_saml_metadata(&request.saml_metadata_document)?;

        // Build SAML provider
        let mut provider = builder::build_saml_provider(
            request.name,
            request.saml_metadata_document.clone(),
            context,
        )?;

        // Extract validity if present
        if let Ok(Some(valid_until)) =
            operations::extract_saml_validity(&request.saml_metadata_document)
        {
            provider = builder::set_saml_valid_until(provider, valid_until);
        }

        // Add tags
        if let Some(tags) = request.tags {
            provider = builder::add_saml_tags(provider, tags);
        }

        // Persist
        let mut store = self.store.write().unwrap();
        store.create_saml_provider(provider).await
    }

    /// Get a SAML provider by ARN
    pub async fn get_saml_provider(&self, arn: &str) -> Result<Option<SamlProvider>> {
        let store = self.store.read().unwrap();
        store.get_saml_provider(arn).await
    }

    /// Update a SAML provider's metadata
    pub async fn update_saml_provider(
        &self,
        request: UpdateSAMLProviderRequest,
    ) -> Result<SamlProvider> {
        // Validate metadata
        operations::validate_saml_metadata(&request.saml_metadata_document)?;

        // Get existing provider
        let existing = {
            let store = self.store.read().unwrap();
            store.get_saml_provider(&request.arn).await?
        };

        let existing = existing.ok_or_else(|| AmiError::ResourceNotFound {
            resource: format!("SamlProvider: {}", request.arn),
        })?;

        // Update metadata
        let mut updated =
            builder::update_saml_metadata(existing, request.saml_metadata_document.clone());

        // Extract and update validity if present
        if let Ok(Some(valid_until)) =
            operations::extract_saml_validity(&request.saml_metadata_document)
        {
            updated = builder::set_saml_valid_until(updated, valid_until);
        }

        // Persist
        let mut store = self.store.write().unwrap();
        store.update_saml_provider(updated).await
    }

    /// Delete a SAML provider
    pub async fn delete_saml_provider(&self, arn: &str) -> Result<()> {
        let mut store = self.store.write().unwrap();
        store.delete_saml_provider(arn).await
    }

    /// List SAML providers
    pub async fn list_saml_providers(
        &self,
        request: ListSAMLProvidersRequest,
    ) -> Result<(Vec<SamlProvider>, bool, Option<String>)> {
        let store = self.store.read().unwrap();
        store.list_saml_providers(request.pagination.as_ref()).await
    }

    // ===========================
    // OIDC Provider Operations
    // ===========================

    /// Create a new OIDC provider
    pub async fn create_oidc_provider(
        &self,
        context: &WamiContext,
        request: CreateOpenIDConnectProviderRequest,
    ) -> Result<OidcProvider> {
        // Validate URL
        operations::validate_oidc_url(&request.url)?;

        // Validate client IDs
        operations::validate_client_id_list(&request.client_id_list)?;

        // Validate thumbprints
        operations::validate_thumbprint_list(&request.thumbprint_list)?;

        // Build OIDC provider
        let mut provider = builder::build_oidc_provider(
            request.url,
            request.client_id_list,
            request.thumbprint_list,
            context,
        )?;

        // Add tags
        if let Some(tags) = request.tags {
            provider = builder::add_oidc_tags(provider, tags);
        }

        // Persist
        let mut store = self.store.write().unwrap();
        store.create_oidc_provider(provider).await
    }

    /// Get an OIDC provider by ARN
    pub async fn get_oidc_provider(&self, arn: &str) -> Result<Option<OidcProvider>> {
        let store = self.store.read().unwrap();
        store.get_oidc_provider(arn).await
    }

    /// Update an OIDC provider's thumbprints
    pub async fn update_oidc_thumbprints(
        &self,
        request: UpdateOpenIDConnectProviderThumbprintRequest,
    ) -> Result<OidcProvider> {
        // Validate thumbprints
        operations::validate_thumbprint_list(&request.thumbprint_list)?;

        // Get existing provider
        let existing = {
            let store = self.store.read().unwrap();
            store.get_oidc_provider(&request.arn).await?
        };

        let existing = existing.ok_or_else(|| AmiError::ResourceNotFound {
            resource: format!("OidcProvider: {}", request.arn),
        })?;

        // Update thumbprints
        let updated = builder::update_thumbprints(existing, request.thumbprint_list);

        // Persist
        let mut store = self.store.write().unwrap();
        store.update_oidc_provider(updated).await
    }

    /// Add a client ID to an OIDC provider
    pub async fn add_client_id(
        &self,
        request: AddClientIDToOpenIDConnectProviderRequest,
    ) -> Result<OidcProvider> {
        // Get existing provider
        let existing = {
            let store = self.store.read().unwrap();
            store.get_oidc_provider(&request.arn).await?
        };

        let existing = existing.ok_or_else(|| AmiError::ResourceNotFound {
            resource: format!("OidcProvider: {}", request.arn),
        })?;

        // Add client ID
        let updated = builder::add_client_id(existing, request.client_id);

        // Persist
        let mut store = self.store.write().unwrap();
        store.update_oidc_provider(updated).await
    }

    /// Remove a client ID from an OIDC provider
    pub async fn remove_client_id(
        &self,
        request: RemoveClientIDFromOpenIDConnectProviderRequest,
    ) -> Result<OidcProvider> {
        // Get existing provider
        let existing = {
            let store = self.store.read().unwrap();
            store.get_oidc_provider(&request.arn).await?
        };

        let existing = existing.ok_or_else(|| AmiError::ResourceNotFound {
            resource: format!("OidcProvider: {}", request.arn),
        })?;

        // Remove client ID
        let updated = builder::remove_client_id(existing, &request.client_id);

        // Persist
        let mut store = self.store.write().unwrap();
        store.update_oidc_provider(updated).await
    }

    /// Delete an OIDC provider
    pub async fn delete_oidc_provider(&self, arn: &str) -> Result<()> {
        let mut store = self.store.write().unwrap();
        store.delete_oidc_provider(arn).await
    }

    /// List OIDC providers
    pub async fn list_oidc_providers(
        &self,
        request: ListOpenIDConnectProvidersRequest,
    ) -> Result<(Vec<OidcProvider>, bool, Option<String>)> {
        let store = self.store.read().unwrap();
        store.list_oidc_providers(request.pagination.as_ref()).await
    }

    // ===========================
    // Tagging Operations
    // ===========================

    /// Tag an identity provider (SAML or OIDC)
    pub async fn tag_identity_provider(&self, arn: &str, tags: Vec<Tag>) -> Result<()> {
        let mut store = self.store.write().unwrap();
        store.tag_identity_provider(arn, tags).await
    }

    /// List tags for an identity provider
    pub async fn list_identity_provider_tags(&self, arn: &str) -> Result<Vec<Tag>> {
        let store = self.store.read().unwrap();
        store.list_identity_provider_tags(arn).await
    }

    /// Untag an identity provider
    pub async fn untag_identity_provider(&self, arn: &str, tag_keys: Vec<String>) -> Result<()> {
        let mut store = self.store.write().unwrap();
        store.untag_identity_provider(arn, tag_keys).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::store::memory::InMemoryWamiStore;

    fn test_context() -> WamiContext {
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("root"))
            .caller_arn(
                WamiArn::builder()
                    .service(crate::arn::Service::Iam)
                    .tenant_path(TenantPath::single("root"))
                    .wami_instance("123456789012")
                    .resource("user", "test-user")
                    .build()
                    .unwrap(),
            )
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_saml_provider_service() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        let service = IdentityProviderService::new(store);
        let context = test_context();

        let metadata = r#"<?xml version="1.0"?>
            <EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata">
                <IDPSSODescriptor />
            </EntityDescriptor>"#;

        // Create
        let request = CreateSAMLProviderRequest {
            name: "TestSAML".to_string(),
            saml_metadata_document: metadata.to_string(),
            tags: None,
        };
        let created = service
            .create_saml_provider(&context, request)
            .await
            .unwrap();
        assert_eq!(created.saml_provider_name, "TestSAML");

        // Get
        let retrieved = service.get_saml_provider(&created.arn).await.unwrap();
        assert!(retrieved.is_some());

        // Update
        let update_req = UpdateSAMLProviderRequest {
            arn: created.arn.clone(),
            saml_metadata_document: metadata.to_string(),
        };
        let updated = service.update_saml_provider(update_req).await.unwrap();
        assert_eq!(updated.arn, created.arn);

        // List
        let (providers, _, _) = service
            .list_saml_providers(ListSAMLProvidersRequest::default())
            .await
            .unwrap();
        assert_eq!(providers.len(), 1);

        // Delete
        service.delete_saml_provider(&created.arn).await.unwrap();
        let after_delete = service.get_saml_provider(&created.arn).await.unwrap();
        assert!(after_delete.is_none());
    }

    #[tokio::test]
    async fn test_oidc_provider_service() {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        let service = IdentityProviderService::new(store);
        let context = test_context();

        // Create
        let request = CreateOpenIDConnectProviderRequest {
            url: "https://accounts.google.com".to_string(),
            client_id_list: vec!["client-123".to_string()],
            thumbprint_list: vec!["0123456789abcdef0123456789abcdef01234567".to_string()],
            tags: None,
        };
        let created = service
            .create_oidc_provider(&context, request)
            .await
            .unwrap();
        assert_eq!(created.url, "https://accounts.google.com");

        // Get
        let retrieved = service.get_oidc_provider(&created.arn).await.unwrap();
        assert!(retrieved.is_some());

        // Update thumbprints
        let update_req = UpdateOpenIDConnectProviderThumbprintRequest {
            arn: created.arn.clone(),
            thumbprint_list: vec!["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string()],
        };
        let updated = service.update_oidc_thumbprints(update_req).await.unwrap();
        assert_eq!(updated.thumbprint_list.len(), 1);

        // Add client ID
        let add_req = AddClientIDToOpenIDConnectProviderRequest {
            arn: created.arn.clone(),
            client_id: "client-456".to_string(),
        };
        let with_client = service.add_client_id(add_req).await.unwrap();
        assert_eq!(with_client.client_id_list.len(), 2);

        // Remove client ID
        let remove_req = RemoveClientIDFromOpenIDConnectProviderRequest {
            arn: created.arn.clone(),
            client_id: "client-123".to_string(),
        };
        let without_client = service.remove_client_id(remove_req).await.unwrap();
        assert_eq!(without_client.client_id_list.len(), 1);

        // List
        let (providers, _, _) = service
            .list_oidc_providers(ListOpenIDConnectProvidersRequest::default())
            .await
            .unwrap();
        assert_eq!(providers.len(), 1);

        // Delete
        service.delete_oidc_provider(&created.arn).await.unwrap();
        let after_delete = service.get_oidc_provider(&created.arn).await.unwrap();
        assert!(after_delete.is_none());
    }
}
