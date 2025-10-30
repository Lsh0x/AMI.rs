//! SAML and OIDC identity provider management

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

// Re-export types
pub use model::{OidcProvider, SamlProvider};
pub use requests::{
    AddClientIDToOpenIDConnectProviderRequest, CreateOpenIDConnectProviderRequest,
    CreateSAMLProviderRequest, DeleteOpenIDConnectProviderRequest, DeleteSAMLProviderRequest,
    GetOpenIDConnectProviderRequest, GetSAMLProviderRequest, ListIdentityProviderTagsRequest,
    ListOpenIDConnectProvidersRequest, ListSAMLProvidersRequest,
    RemoveClientIDFromOpenIDConnectProviderRequest, TagIdentityProviderRequest,
    UntagIdentityProviderRequest, UpdateOpenIDConnectProviderThumbprintRequest,
    UpdateSAMLProviderRequest,
};
