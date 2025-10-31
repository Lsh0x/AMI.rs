//! Identity Provider Builder Functions
//!
//! Pure functions for building and manipulating identity provider resources.

use super::model::{OidcProvider, SamlProvider};
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::types::Tag;
use chrono::Utc;

/// Build a new SAML provider (pure function)
///
/// Creates a SAML provider with the given name and metadata document.
#[allow(clippy::result_large_err)]
pub fn build_saml_provider(
    name: String,
    saml_metadata_document: String,
    context: &WamiContext,
) -> Result<SamlProvider> {
    // Generate AWS-compatible ARN
    let arn = format!(
        "arn:aws:iam::{}:saml-provider/{}",
        context.instance_id(),
        name
    );

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("saml-provider", &name)
        .build()?;

    Ok(SamlProvider {
        arn,
        saml_provider_name: name,
        saml_metadata_document,
        create_date: Utc::now(),
        valid_until: None,
        tags: Vec::new(),
        wami_arn: wami_arn.to_string(),
        providers: Vec::new(),
        tenant_id: None,
        usage_count: 0,
    })
}

/// Build a new OIDC provider (pure function)
///
/// Creates an OIDC provider with the given URL, client IDs, and thumbprints.
#[allow(clippy::result_large_err)]
pub fn build_oidc_provider(
    url: String,
    client_id_list: Vec<String>,
    thumbprint_list: Vec<String>,
    context: &WamiContext,
) -> Result<OidcProvider> {
    // For OIDC, the URL serves as the "name" in the ARN
    // Strip https:// prefix for the ARN name
    let arn_name = url.trim_start_matches("https://");

    // Generate AWS-compatible ARN
    let arn = format!(
        "arn:aws:iam::{}:oidc-provider/{}",
        context.instance_id(),
        arn_name
    );

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("oidc-provider", arn_name)
        .build()?;

    Ok(OidcProvider {
        arn,
        url,
        client_id_list,
        thumbprint_list,
        create_date: Utc::now(),
        tags: Vec::new(),
        wami_arn: wami_arn.to_string(),
        providers: Vec::new(),
        tenant_id: None,
        usage_count: 0,
    })
}

/// Update SAML metadata document (pure function)
pub fn update_saml_metadata(
    mut provider: SamlProvider,
    saml_metadata_document: String,
) -> SamlProvider {
    provider.saml_metadata_document = saml_metadata_document;
    provider
}

/// Set the valid_until date for a SAML provider (pure function)
pub fn set_saml_valid_until(
    mut provider: SamlProvider,
    valid_until: chrono::DateTime<chrono::Utc>,
) -> SamlProvider {
    provider.valid_until = Some(valid_until);
    provider
}

/// Add a client ID to an OIDC provider (pure function)
pub fn add_client_id(mut provider: OidcProvider, client_id: String) -> OidcProvider {
    if !provider.client_id_list.contains(&client_id) {
        provider.client_id_list.push(client_id);
    }
    provider
}

/// Remove a client ID from an OIDC provider (pure function)
pub fn remove_client_id(mut provider: OidcProvider, client_id: &str) -> OidcProvider {
    provider.client_id_list.retain(|id| id != client_id);
    provider
}

/// Update thumbprints for an OIDC provider (pure function)
pub fn update_thumbprints(
    mut provider: OidcProvider,
    thumbprint_list: Vec<String>,
) -> OidcProvider {
    provider.thumbprint_list = thumbprint_list;
    provider
}

/// Add tags to a SAML provider (pure function)
pub fn add_saml_tags(mut provider: SamlProvider, tags: Vec<Tag>) -> SamlProvider {
    for tag in tags {
        // Remove existing tag with same key
        provider.tags.retain(|t| t.key != tag.key);
        provider.tags.push(tag);
    }
    provider
}

/// Add tags to an OIDC provider (pure function)
pub fn add_oidc_tags(mut provider: OidcProvider, tags: Vec<Tag>) -> OidcProvider {
    for tag in tags {
        // Remove existing tag with same key
        provider.tags.retain(|t| t.key != tag.key);
        provider.tags.push(tag);
    }
    provider
}

/// Increment usage count for SAML provider (pure function)
pub fn increment_saml_usage(mut provider: SamlProvider) -> SamlProvider {
    provider.usage_count = provider.usage_count.saturating_add(1);
    provider
}

/// Decrement usage count for SAML provider (pure function)
pub fn decrement_saml_usage(mut provider: SamlProvider) -> SamlProvider {
    provider.usage_count = provider.usage_count.saturating_sub(1);
    provider
}

/// Increment usage count for OIDC provider (pure function)
pub fn increment_oidc_usage(mut provider: OidcProvider) -> OidcProvider {
    provider.usage_count = provider.usage_count.saturating_add(1);
    provider
}

/// Decrement usage count for OIDC provider (pure function)
pub fn decrement_oidc_usage(mut provider: OidcProvider) -> OidcProvider {
    provider.usage_count = provider.usage_count.saturating_sub(1);
    provider
}

/// Set tenant ID for SAML provider (pure function)
pub fn set_saml_tenant(
    mut provider: SamlProvider,
    tenant_id: crate::wami::tenant::TenantId,
) -> SamlProvider {
    provider.tenant_id = Some(tenant_id);
    provider
}

/// Set tenant ID for OIDC provider (pure function)
pub fn set_oidc_tenant(
    mut provider: OidcProvider,
    tenant_id: crate::wami::tenant::TenantId,
) -> OidcProvider {
    provider.tenant_id = Some(tenant_id);
    provider
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};

    fn test_context() -> WamiContext {
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(0))
            .caller_arn(
                WamiArn::builder()
                    .service(Service::Iam)
                    .tenant_path(TenantPath::single(0))
                    .wami_instance("123456789012")
                    .resource("user", "test-user")
                    .build()
                    .unwrap(),
            )
            .is_root(false)
            .build()
            .unwrap()
    }

    #[test]
    fn test_build_saml_provider() {
        let context = test_context();
        let saml = build_saml_provider(
            "TestProvider".to_string(),
            "<EntityDescriptor />".to_string(),
            &context,
        )
        .unwrap();

        assert_eq!(saml.saml_provider_name, "TestProvider");
        assert!(saml.arn.contains("saml-provider"));
        assert!(saml.arn.contains("TestProvider"));
        assert_eq!(saml.usage_count, 0);
        assert!(saml.valid_until.is_none());
    }

    #[test]
    fn test_build_oidc_provider() {
        let context = test_context();
        let oidc = build_oidc_provider(
            "https://accounts.google.com".to_string(),
            vec!["client-id-123".to_string()],
            vec!["0123456789abcdef0123456789abcdef01234567".to_string()],
            &context,
        )
        .unwrap();

        assert_eq!(oidc.url, "https://accounts.google.com");
        assert!(oidc.arn.contains("oidc-provider"));
        assert!(oidc.arn.contains("accounts.google.com"));
        assert_eq!(oidc.client_id_list.len(), 1);
        assert_eq!(oidc.thumbprint_list.len(), 1);
        assert_eq!(oidc.usage_count, 0);
    }

    #[test]
    fn test_update_saml_metadata() {
        let context = test_context();
        let saml = build_saml_provider("Test".to_string(), "old".to_string(), &context).unwrap();

        let updated = update_saml_metadata(saml, "new metadata".to_string());
        assert_eq!(updated.saml_metadata_document, "new metadata");
    }

    #[test]
    fn test_add_remove_client_id() {
        let context = test_context();
        let oidc = build_oidc_provider(
            "https://example.com".to_string(),
            vec!["client1".to_string()],
            vec!["0123456789abcdef0123456789abcdef01234567".to_string()],
            &context,
        )
        .unwrap();

        let with_client = add_client_id(oidc.clone(), "client2".to_string());
        assert_eq!(with_client.client_id_list.len(), 2);

        let without_client = remove_client_id(with_client, "client1");
        assert_eq!(without_client.client_id_list.len(), 1);
        assert_eq!(without_client.client_id_list[0], "client2");
    }

    #[test]
    fn test_usage_tracking() {
        let context = test_context();
        let saml = build_saml_provider("Test".to_string(), "meta".to_string(), &context).unwrap();

        let incremented = increment_saml_usage(saml);
        assert_eq!(incremented.usage_count, 1);

        let incremented2 = increment_saml_usage(incremented);
        assert_eq!(incremented2.usage_count, 2);

        let decremented = decrement_saml_usage(incremented2);
        assert_eq!(decremented.usage_count, 1);
    }

    #[test]
    fn test_update_thumbprints() {
        let context = test_context();
        let oidc = build_oidc_provider(
            "https://example.com".to_string(),
            vec![],
            vec!["0123456789abcdef0123456789abcdef01234567".to_string()],
            &context,
        )
        .unwrap();

        let new_thumbprints = vec![
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
            "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string(),
        ];

        let updated = update_thumbprints(oidc, new_thumbprints.clone());
        assert_eq!(updated.thumbprint_list.len(), 2);
        assert_eq!(updated.thumbprint_list, new_thumbprints);
    }
}
