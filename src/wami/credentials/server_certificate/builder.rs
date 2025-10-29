//! Server Certificate Builder

use super::model::*;
use crate::provider::{CloudProvider, ResourceType};
use chrono::Utc;

/// Build a new ServerCertificate resource
pub fn build_server_certificate(
    server_certificate_name: String,
    certificate_body: String,
    certificate_chain: Option<String>,
    path: String,
    tags: Vec<crate::types::Tag>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> ServerCertificate {
    let cert_id = provider.generate_resource_id(ResourceType::ServerCertificate);
    let arn = provider.generate_resource_identifier(
        ResourceType::ServerCertificate,
        account_id,
        &path,
        &server_certificate_name,
    );

    let wami_arn = provider.generate_wami_arn(
        ResourceType::ServerCertificate,
        account_id,
        &path,
        &server_certificate_name,
    );

    let metadata = ServerCertificateMetadata {
        path,
        server_certificate_name,
        arn,
        server_certificate_id: cert_id,
        upload_date: Utc::now(),
        expiration: None, // Would need to parse cert to get actual expiration
    };

    ServerCertificate {
        server_certificate_metadata: metadata,
        certificate_body,
        certificate_chain,
        tags,
        wami_arn,
        providers: Vec::new(),
    }
}
