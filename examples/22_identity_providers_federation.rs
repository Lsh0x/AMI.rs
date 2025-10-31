//! Example 22: Identity Providers for Federation
//!
//! Demonstrates setting up SAML and OIDC identity providers for federated authentication.
//!
//! This example shows:
//! - Creating SAML providers for enterprise IdPs (Okta, Azure AD)
//! - Creating OIDC providers for modern IdPs (Google, Auth0)
//! - Updating provider configurations (thumbprints, client IDs)
//! - Listing and managing providers
//! - Tagging providers for organization
//! - Tracking provider usage
//!
//! Run with: cargo run --example 22_identity_providers_federation

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::IdentityProviderService;
use wami::store::memory::InMemoryWamiStore;
use wami::types::Tag;
use wami::wami::identity::identity_provider::{
    AddClientIDToOpenIDConnectProviderRequest, CreateOpenIDConnectProviderRequest,
    CreateSAMLProviderRequest, ListOpenIDConnectProvidersRequest, ListSAMLProvidersRequest,
    RemoveClientIDFromOpenIDConnectProviderRequest, UpdateOpenIDConnectProviderThumbprintRequest,
    UpdateSAMLProviderRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== WAMI Example 22: Identity Providers for Federation ===\n");

    // Create in-memory store
    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create context
    let context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single("root"))
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single("root"))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Create identity provider service
    let service = IdentityProviderService::new(store.clone());

    println!("📋 Setting up federated authentication with SAML and OIDC providers\n");

    // ===========================
    // Part 1: SAML Providers
    // ===========================

    println!("🔐 Part 1: SAML Providers (Enterprise Federation)\n");

    // Create Okta SAML provider
    println!("Creating Okta SAML provider...");
    let okta_metadata = r#"<?xml version="1.0" encoding="UTF-8"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata"
                  entityID="http://www.okta.com/exampleid"
                  validUntil="2025-12-31T23:59:59Z">
    <IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
        <KeyDescriptor use="signing">
            <ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
                <ds:X509Data>
                    <ds:X509Certificate>MIIDpDCCAoygAwIBAgIGAWk...</ds:X509Certificate>
                </ds:X509Data>
            </ds:KeyInfo>
        </KeyDescriptor>
        <SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                             Location="https://example.okta.com/app/exampleid/sso/saml"/>
    </IDPSSODescriptor>
</EntityDescriptor>"#;

    let okta_request = CreateSAMLProviderRequest {
        name: "OktaEnterpriseProvider".to_string(),
        saml_metadata_document: okta_metadata.to_string(),
        tags: Some(vec![
            Tag {
                key: "Environment".to_string(),
                value: "Production".to_string(),
            },
            Tag {
                key: "IdP".to_string(),
                value: "Okta".to_string(),
            },
        ]),
    };

    let okta_provider = service.create_saml_provider(&context, okta_request).await?;
    println!("✅ Created Okta SAML provider: {}", okta_provider.arn);
    println!("   Name: {}", okta_provider.saml_provider_name);
    if let Some(valid_until) = okta_provider.valid_until {
        println!("   Valid until: {}", valid_until);
    }
    println!("   Tags: {:?}\n", okta_provider.tags);

    // Create Azure AD SAML provider
    println!("Creating Azure AD SAML provider...");
    let azure_metadata = r#"<?xml version="1.0" encoding="UTF-8"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata"
                  entityID="https://sts.windows.net/tenant-id/">
    <IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
        <SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                             Location="https://login.microsoftonline.com/tenant-id/saml2"/>
    </IDPSSODescriptor>
</EntityDescriptor>"#;

    let azure_request = CreateSAMLProviderRequest {
        name: "AzureAD-Enterprise".to_string(),
        saml_metadata_document: azure_metadata.to_string(),
        tags: Some(vec![Tag {
            key: "IdP".to_string(),
            value: "AzureAD".to_string(),
        }]),
    };

    let azure_provider = service
        .create_saml_provider(&context, azure_request)
        .await?;
    println!("✅ Created Azure AD SAML provider: {}", azure_provider.arn);
    println!("   Name: {}\n", azure_provider.saml_provider_name);

    // Update SAML provider metadata (certificate rotation scenario)
    println!("Updating Okta SAML provider metadata (certificate rotation)...");
    let updated_metadata =
        okta_metadata.replace("MIIDpDCCAoygAwIBAgIGAWk...", "MIIDnEWCERTIFICATE...");
    let update_request = UpdateSAMLProviderRequest {
        arn: okta_provider.arn.clone(),
        saml_metadata_document: updated_metadata,
    };
    let updated_okta = service.update_saml_provider(update_request).await?;
    println!("✅ Updated Okta SAML provider metadata\n");

    // List SAML providers
    println!("Listing all SAML providers:");
    let (saml_providers, is_truncated, _) = service
        .list_saml_providers(ListSAMLProvidersRequest::default())
        .await?;
    for (i, provider) in saml_providers.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            i + 1,
            provider.saml_provider_name,
            provider.arn
        );
    }
    println!("  Total: {} provider(s)", saml_providers.len());
    if is_truncated {
        println!("  (more available)");
    }
    println!();

    // ===========================
    // Part 2: OIDC Providers
    // ===========================

    println!("🌐 Part 2: OIDC Providers (Modern Federation)\n");

    // Create Google OIDC provider
    println!("Creating Google OIDC provider...");
    let google_request = CreateOpenIDConnectProviderRequest {
        url: "https://accounts.google.com".to_string(),
        client_id_list: vec!["1234567890-abcdefghijklmnop.apps.googleusercontent.com".to_string()],
        thumbprint_list: vec![
            // Google's certificate thumbprint (example)
            "c3846bf24b9e93ca64274c0ec67c1ecc5e024ffb".to_string(),
        ],
        tags: Some(vec![
            Tag {
                key: "IdP".to_string(),
                value: "Google".to_string(),
            },
            Tag {
                key: "Usage".to_string(),
                value: "WebApp".to_string(),
            },
        ]),
    };

    let google_provider = service
        .create_oidc_provider(&context, google_request)
        .await?;
    println!("✅ Created Google OIDC provider: {}", google_provider.arn);
    println!("   URL: {}", google_provider.url);
    println!("   Client IDs: {:?}", google_provider.client_id_list);
    println!("   Thumbprints: {:?}\n", google_provider.thumbprint_list);

    // Create Auth0 OIDC provider
    println!("Creating Auth0 OIDC provider...");
    let auth0_request = CreateOpenIDConnectProviderRequest {
        url: "https://myapp.us.auth0.com".to_string(),
        client_id_list: vec!["Auth0ClientIdExample123456".to_string()],
        thumbprint_list: vec!["9e99a48a9960b14926bb7f3b02e22da2b0ab7280".to_string()],
        tags: Some(vec![Tag {
            key: "IdP".to_string(),
            value: "Auth0".to_string(),
        }]),
    };

    let auth0_provider = service
        .create_oidc_provider(&context, auth0_request)
        .await?;
    println!("✅ Created Auth0 OIDC provider: {}", auth0_provider.arn);
    println!("   URL: {}\n", auth0_provider.url);

    // ===========================
    // Part 3: Managing OIDC Providers
    // ===========================

    println!("🔧 Part 3: Managing OIDC Provider Configuration\n");

    // Add additional client ID to Google provider
    println!("Adding additional client ID to Google provider...");
    let add_client_request = AddClientIDToOpenIDConnectProviderRequest {
        arn: google_provider.arn.clone(),
        client_id: "0987654321-zyxwvutsrqpon.apps.googleusercontent.com".to_string(),
    };
    let updated_google = service.add_client_id(add_client_request).await?;
    println!(
        "✅ Added client ID. Total client IDs: {}",
        updated_google.client_id_list.len()
    );
    println!("   Client IDs: {:?}\n", updated_google.client_id_list);

    // Update thumbprints (certificate rotation)
    println!("Updating Auth0 thumbprints (certificate rotation)...");
    let update_thumbprint_request = UpdateOpenIDConnectProviderThumbprintRequest {
        arn: auth0_provider.arn.clone(),
        thumbprint_list: vec![
            "9e99a48a9960b14926bb7f3b02e22da2b0ab7280".to_string(),
            "a053375bfe84e8b748782c7cee15827a6af5a405".to_string(), // New certificate
        ],
    };
    let updated_auth0 = service
        .update_oidc_thumbprints(update_thumbprint_request)
        .await?;
    println!(
        "✅ Updated thumbprints. Total: {}",
        updated_auth0.thumbprint_list.len()
    );
    println!("   Thumbprints: {:?}\n", updated_auth0.thumbprint_list);

    // Remove a client ID
    println!("Removing original Google client ID...");
    let remove_client_request = RemoveClientIDFromOpenIDConnectProviderRequest {
        arn: google_provider.arn.clone(),
        client_id: "1234567890-abcdefghijklmnop.apps.googleusercontent.com".to_string(),
    };
    let final_google = service.remove_client_id(remove_client_request).await?;
    println!(
        "✅ Removed client ID. Remaining client IDs: {}",
        final_google.client_id_list.len()
    );
    println!("   Client IDs: {:?}\n", final_google.client_id_list);

    // List OIDC providers
    println!("Listing all OIDC providers:");
    let (oidc_providers, is_truncated, _) = service
        .list_oidc_providers(ListOpenIDConnectProvidersRequest::default())
        .await?;
    for (i, provider) in oidc_providers.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, provider.url, provider.arn);
    }
    println!("  Total: {} provider(s)", oidc_providers.len());
    if is_truncated {
        println!("  (more available)");
    }
    println!();

    // ===========================
    // Part 4: Tagging Operations
    // ===========================

    println!("🏷️  Part 4: Tagging Identity Providers\n");

    // Add tags to a provider
    println!("Adding tags to Google OIDC provider...");
    service
        .tag_identity_provider(
            &google_provider.arn,
            vec![
                Tag {
                    key: "CostCenter".to_string(),
                    value: "Engineering".to_string(),
                },
                Tag {
                    key: "Compliance".to_string(),
                    value: "SOC2".to_string(),
                },
            ],
        )
        .await?;
    println!("✅ Added tags\n");

    // List tags
    println!("Listing tags for Google provider:");
    let tags = service
        .list_identity_provider_tags(&google_provider.arn)
        .await?;
    for tag in &tags {
        println!("  - {}: {}", tag.key, tag.value);
    }
    println!();

    // Remove a tag
    println!("Removing 'Usage' tag...");
    service
        .untag_identity_provider(&google_provider.arn, vec!["Usage".to_string()])
        .await?;
    println!("✅ Removed tag\n");

    // ===========================
    // Part 5: Usage Tracking
    // ===========================

    println!("📊 Part 5: Provider Usage Tracking\n");

    // Retrieve provider and check usage
    let google_final = service.get_oidc_provider(&google_provider.arn).await?;
    if let Some(provider) = google_final {
        println!("Google OIDC Provider:");
        println!("  URL: {}", provider.url);
        println!("  Usage count: {} principals", provider.usage_count);
        println!("  Created: {}", provider.create_date);
        println!("  Tags: {} tag(s)", provider.tags.len());
    }
    println!();

    let okta_final = service.get_saml_provider(&updated_okta.arn).await?;
    if let Some(provider) = okta_final {
        println!("Okta SAML Provider:");
        println!("  Name: {}", provider.saml_provider_name);
        println!("  Usage count: {} principals", provider.usage_count);
        println!("  Created: {}", provider.create_date);
        if let Some(valid_until) = provider.valid_until {
            println!("  Valid until: {}", valid_until);
        }
    }
    println!();

    // ===========================
    // Part 6: Cleanup
    // ===========================

    println!("🧹 Part 6: Cleanup (Optional)\n");

    println!("Note: In production, you'd typically keep identity providers configured.");
    println!("For this demo, we'll list what we've created:\n");

    let (all_saml, _, _) = service
        .list_saml_providers(ListSAMLProvidersRequest::default())
        .await?;
    let (all_oidc, _, _) = service
        .list_oidc_providers(ListOpenIDConnectProvidersRequest::default())
        .await?;

    println!("Summary:");
    println!("  - {} SAML providers configured", all_saml.len());
    println!("  - {} OIDC providers configured", all_oidc.len());
    println!(
        "  - Total: {} identity providers",
        all_saml.len() + all_oidc.len()
    );
    println!();

    // Uncomment to actually delete providers:
    // println!("Cleaning up providers...");
    // service.delete_saml_provider(&okta_provider.arn).await?;
    // service.delete_saml_provider(&azure_provider.arn).await?;
    // service.delete_oidc_provider(&google_provider.arn).await?;
    // service.delete_oidc_provider(&auth0_provider.arn).await?;
    // println!("✅ All providers deleted\n");

    println!("=== Example 22 Complete ===");
    println!();
    println!("Key Takeaways:");
    println!("  1. SAML providers are ideal for enterprise SSO (Okta, Azure AD)");
    println!("  2. OIDC providers are perfect for modern web apps (Google, Auth0)");
    println!("  3. Both support certificate rotation via update operations");
    println!("  4. Tagging helps organize providers by environment, compliance, etc.");
    println!("  5. Usage tracking shows which providers are actively used");
    println!();
    println!("Next Steps:");
    println!("  - Configure trust relationships in your roles to use these providers");
    println!("  - Set up AssumeRoleWithSAML or AssumeRoleWithWebIdentity for authentication");
    println!("  - Monitor provider usage and update certificates before expiration");

    Ok(())
}
