//! Identity Provider Request Types
//!
//! Request and response structures for SAML and OIDC provider operations.

use crate::types::{PaginationParams, Tag};
use serde::{Deserialize, Serialize};

// ===========================
// SAML Provider Requests
// ===========================

/// Request to create a new SAML provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSAMLProviderRequest {
    /// The name of the SAML provider (1-128 characters)
    pub name: String,
    /// The SAML metadata document (XML format)
    pub saml_metadata_document: String,
    /// Optional tags to attach to the provider
    pub tags: Option<Vec<Tag>>,
}

/// Request to update a SAML provider's metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSAMLProviderRequest {
    /// The ARN of the SAML provider to update
    pub arn: String,
    /// The new SAML metadata document (XML format)
    pub saml_metadata_document: String,
}

/// Request to get a SAML provider's details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSAMLProviderRequest {
    /// The ARN of the SAML provider
    pub arn: String,
}

/// Request to delete a SAML provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSAMLProviderRequest {
    /// The ARN of the SAML provider to delete
    pub arn: String,
}

/// Request to list SAML providers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListSAMLProvidersRequest {
    /// Optional pagination parameters
    pub pagination: Option<PaginationParams>,
}

// ===========================
// OIDC Provider Requests
// ===========================

/// Request to create a new OIDC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOpenIDConnectProviderRequest {
    /// The URL of the OIDC provider (must use HTTPS)
    pub url: String,
    /// List of client IDs (audience) allowed to use this provider
    pub client_id_list: Vec<String>,
    /// List of server certificate thumbprints (SHA-1 fingerprints)
    pub thumbprint_list: Vec<String>,
    /// Optional tags to attach to the provider
    pub tags: Option<Vec<Tag>>,
}

/// Request to update an OIDC provider's thumbprints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOpenIDConnectProviderThumbprintRequest {
    /// The ARN of the OIDC provider
    pub arn: String,
    /// The new list of server certificate thumbprints
    pub thumbprint_list: Vec<String>,
}

/// Request to add a client ID to an OIDC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddClientIDToOpenIDConnectProviderRequest {
    /// The ARN of the OIDC provider
    pub arn: String,
    /// The client ID to add
    pub client_id: String,
}

/// Request to remove a client ID from an OIDC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveClientIDFromOpenIDConnectProviderRequest {
    /// The ARN of the OIDC provider
    pub arn: String,
    /// The client ID to remove
    pub client_id: String,
}

/// Request to get an OIDC provider's details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOpenIDConnectProviderRequest {
    /// The ARN of the OIDC provider
    pub arn: String,
}

/// Request to delete an OIDC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOpenIDConnectProviderRequest {
    /// The ARN of the OIDC provider to delete
    pub arn: String,
}

/// Request to list OIDC providers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListOpenIDConnectProvidersRequest {
    /// Optional pagination parameters
    pub pagination: Option<PaginationParams>,
}

// ===========================
// Tagging Requests
// ===========================

/// Request to tag an identity provider (SAML or OIDC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagIdentityProviderRequest {
    /// The ARN of the identity provider
    pub arn: String,
    /// The tags to add
    pub tags: Vec<Tag>,
}

/// Request to list tags for an identity provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListIdentityProviderTagsRequest {
    /// The ARN of the identity provider
    pub arn: String,
}

/// Request to untag an identity provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UntagIdentityProviderRequest {
    /// The ARN of the identity provider
    pub arn: String,
    /// The tag keys to remove
    pub tag_keys: Vec<String>,
}
