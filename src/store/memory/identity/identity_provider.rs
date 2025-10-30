//! In-Memory Identity Provider Store Implementation

use crate::error::{AmiError, Result};
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::IdentityProviderStore;
use crate::types::{PaginationParams, Tag};
use crate::wami::identity::identity_provider::{OidcProvider, SamlProvider};
use async_trait::async_trait;

#[async_trait]
impl IdentityProviderStore for InMemoryWamiStore {
    // ===========================
    // SAML Provider Operations
    // ===========================

    async fn create_saml_provider(&mut self, provider: SamlProvider) -> Result<SamlProvider> {
        if self.saml_providers.contains_key(&provider.arn) {
            return Err(AmiError::ResourceExists {
                resource: format!("SamlProvider: {}", provider.arn),
            });
        }

        self.saml_providers
            .insert(provider.arn.clone(), provider.clone());
        Ok(provider)
    }

    async fn get_saml_provider(&self, arn: &str) -> Result<Option<SamlProvider>> {
        Ok(self.saml_providers.get(arn).cloned())
    }

    async fn update_saml_provider(&mut self, provider: SamlProvider) -> Result<SamlProvider> {
        if !self.saml_providers.contains_key(&provider.arn) {
            return Err(AmiError::ResourceNotFound {
                resource: format!("SamlProvider: {}", provider.arn),
            });
        }

        self.saml_providers
            .insert(provider.arn.clone(), provider.clone());
        Ok(provider)
    }

    async fn delete_saml_provider(&mut self, arn: &str) -> Result<()> {
        if self.saml_providers.remove(arn).is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("SamlProvider: {}", arn),
            });
        }
        Ok(())
    }

    async fn list_saml_providers(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<SamlProvider>, bool, Option<String>)> {
        let mut providers: Vec<SamlProvider> = self.saml_providers.values().cloned().collect();

        // Sort by create_date for consistent pagination
        providers.sort_by(|a, b| a.create_date.cmp(&b.create_date));

        // Apply pagination
        let (start_index, max_items) = if let Some(params) = pagination {
            let start = params
                .marker
                .as_ref()
                .and_then(|m| providers.iter().position(|p| p.arn == *m))
                .map(|pos| pos + 1)
                .unwrap_or(0);
            (start, params.max_items.unwrap_or(100) as usize)
        } else {
            (0, 100)
        };

        let end_index = std::cmp::min(start_index + max_items, providers.len());
        let paginated = providers[start_index..end_index].to_vec();
        let is_truncated = end_index < providers.len();
        let next_marker = if is_truncated {
            paginated.last().map(|p| p.arn.clone())
        } else {
            None
        };

        Ok((paginated, is_truncated, next_marker))
    }

    // ===========================
    // OIDC Provider Operations
    // ===========================

    async fn create_oidc_provider(&mut self, provider: OidcProvider) -> Result<OidcProvider> {
        if self.oidc_providers.contains_key(&provider.arn) {
            return Err(AmiError::ResourceExists {
                resource: format!("OidcProvider: {}", provider.arn),
            });
        }

        self.oidc_providers
            .insert(provider.arn.clone(), provider.clone());
        Ok(provider)
    }

    async fn get_oidc_provider(&self, arn: &str) -> Result<Option<OidcProvider>> {
        Ok(self.oidc_providers.get(arn).cloned())
    }

    async fn update_oidc_provider(&mut self, provider: OidcProvider) -> Result<OidcProvider> {
        if !self.oidc_providers.contains_key(&provider.arn) {
            return Err(AmiError::ResourceNotFound {
                resource: format!("OidcProvider: {}", provider.arn),
            });
        }

        self.oidc_providers
            .insert(provider.arn.clone(), provider.clone());
        Ok(provider)
    }

    async fn delete_oidc_provider(&mut self, arn: &str) -> Result<()> {
        if self.oidc_providers.remove(arn).is_none() {
            return Err(AmiError::ResourceNotFound {
                resource: format!("OidcProvider: {}", arn),
            });
        }
        Ok(())
    }

    async fn list_oidc_providers(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<OidcProvider>, bool, Option<String>)> {
        let mut providers: Vec<OidcProvider> = self.oidc_providers.values().cloned().collect();

        // Sort by create_date for consistent pagination
        providers.sort_by(|a, b| a.create_date.cmp(&b.create_date));

        // Apply pagination
        let (start_index, max_items) = if let Some(params) = pagination {
            let start = params
                .marker
                .as_ref()
                .and_then(|m| providers.iter().position(|p| p.arn == *m))
                .map(|pos| pos + 1)
                .unwrap_or(0);
            (start, params.max_items.unwrap_or(100) as usize)
        } else {
            (0, 100)
        };

        let end_index = std::cmp::min(start_index + max_items, providers.len());
        let paginated = providers[start_index..end_index].to_vec();
        let is_truncated = end_index < providers.len();
        let next_marker = if is_truncated {
            paginated.last().map(|p| p.arn.clone())
        } else {
            None
        };

        Ok((paginated, is_truncated, next_marker))
    }

    // ===========================
    // Tagging Operations
    // ===========================

    async fn tag_identity_provider(&mut self, arn: &str, tags: Vec<Tag>) -> Result<()> {
        // Check SAML providers first
        if let Some(mut provider) = self.saml_providers.get(arn).cloned() {
            for tag in tags {
                provider.tags.retain(|t| t.key != tag.key);
                provider.tags.push(tag);
            }
            self.saml_providers.insert(arn.to_string(), provider);
            return Ok(());
        }

        // Then check OIDC providers
        if let Some(mut provider) = self.oidc_providers.get(arn).cloned() {
            for tag in tags {
                provider.tags.retain(|t| t.key != tag.key);
                provider.tags.push(tag);
            }
            self.oidc_providers.insert(arn.to_string(), provider);
            return Ok(());
        }

        Err(AmiError::ResourceNotFound {
            resource: format!("IdentityProvider: {}", arn),
        })
    }

    async fn list_identity_provider_tags(&self, arn: &str) -> Result<Vec<Tag>> {
        // Check SAML providers first
        if let Some(provider) = self.saml_providers.get(arn) {
            return Ok(provider.tags.clone());
        }

        // Then check OIDC providers
        if let Some(provider) = self.oidc_providers.get(arn) {
            return Ok(provider.tags.clone());
        }

        Err(AmiError::ResourceNotFound {
            resource: format!("IdentityProvider: {}", arn),
        })
    }

    async fn untag_identity_provider(&mut self, arn: &str, tag_keys: Vec<String>) -> Result<()> {
        // Check SAML providers first
        if let Some(mut provider) = self.saml_providers.get(arn).cloned() {
            provider.tags.retain(|t| !tag_keys.contains(&t.key));
            self.saml_providers.insert(arn.to_string(), provider);
            return Ok(());
        }

        // Then check OIDC providers
        if let Some(mut provider) = self.oidc_providers.get(arn).cloned() {
            provider.tags.retain(|t| !tag_keys.contains(&t.key));
            self.oidc_providers.insert(arn.to_string(), provider);
            return Ok(());
        }

        Err(AmiError::ResourceNotFound {
            resource: format!("IdentityProvider: {}", arn),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::AwsProvider;
    use crate::wami::identity::identity_provider::builder;

    #[tokio::test]
    async fn test_saml_provider_crud() {
        let mut store = InMemoryWamiStore::default();
        let provider = AwsProvider::new();

        let saml = builder::build_saml_provider(
            "TestProvider".to_string(),
            "<EntityDescriptor />".to_string(),
            &provider,
            "123456789012",
        );

        // Create
        let created = store.create_saml_provider(saml.clone()).await.unwrap();
        assert_eq!(created.saml_provider_name, "TestProvider");

        // Get
        let retrieved = store.get_saml_provider(&created.arn).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().arn, created.arn);

        // Update
        let mut updated = created.clone();
        updated.saml_metadata_document = "new metadata".to_string();
        let updated_result = store.update_saml_provider(updated).await.unwrap();
        assert_eq!(updated_result.saml_metadata_document, "new metadata");

        // Delete
        store.delete_saml_provider(&created.arn).await.unwrap();
        assert!(store
            .get_saml_provider(&created.arn)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_oidc_provider_crud() {
        let mut store = InMemoryWamiStore::default();
        let provider = AwsProvider::new();

        let oidc = builder::build_oidc_provider(
            "https://accounts.google.com".to_string(),
            vec!["client-id".to_string()],
            vec!["0123456789abcdef0123456789abcdef01234567".to_string()],
            &provider,
            "123456789012",
        );

        // Create
        let created = store.create_oidc_provider(oidc.clone()).await.unwrap();
        assert_eq!(created.url, "https://accounts.google.com");

        // Get
        let retrieved = store.get_oidc_provider(&created.arn).await.unwrap();
        assert!(retrieved.is_some());

        // Update
        let mut updated = created.clone();
        updated.client_id_list.push("new-client".to_string());
        let updated_result = store.update_oidc_provider(updated).await.unwrap();
        assert_eq!(updated_result.client_id_list.len(), 2);

        // Delete
        store.delete_oidc_provider(&created.arn).await.unwrap();
        assert!(store
            .get_oidc_provider(&created.arn)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_tagging_operations() {
        let mut store = InMemoryWamiStore::default();
        let provider = AwsProvider::new();

        let saml = builder::build_saml_provider(
            "TagTest".to_string(),
            "<EntityDescriptor />".to_string(),
            &provider,
            "123456789012",
        );

        let created = store.create_saml_provider(saml).await.unwrap();

        // Tag
        let tags = vec![Tag {
            key: "Environment".to_string(),
            value: "Production".to_string(),
        }];
        store
            .tag_identity_provider(&created.arn, tags.clone())
            .await
            .unwrap();

        // List tags
        let listed_tags = store
            .list_identity_provider_tags(&created.arn)
            .await
            .unwrap();
        assert_eq!(listed_tags.len(), 1);
        assert_eq!(listed_tags[0].key, "Environment");

        // Untag
        store
            .untag_identity_provider(&created.arn, vec!["Environment".to_string()])
            .await
            .unwrap();

        let listed_after_untag = store
            .list_identity_provider_tags(&created.arn)
            .await
            .unwrap();
        assert_eq!(listed_after_untag.len(), 0);
    }
}
