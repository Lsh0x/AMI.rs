//! Server Certificate Builder

use super::model::*;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use chrono::Utc;
use uuid::Uuid;

/// Build a new ServerCertificate resource with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_server_certificate(
    server_certificate_name: String,
    certificate_body: String,
    certificate_chain: Option<String>,
    path: String,
    tags: Vec<crate::types::Tag>,
    context: &WamiContext,
) -> Result<ServerCertificate> {
    let cert_id = Uuid::new_v4().to_string();

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("server-certificate", &cert_id)
        .build()?;

    // Generate AWS-compatible ARN
    let arn = format!(
        "arn:aws:iam::{}:server-certificate{}{}",
        context.instance_id(),
        if path == "/" { "" } else { &path },
        server_certificate_name
    );

    let metadata = ServerCertificateMetadata {
        path,
        server_certificate_name,
        arn,
        server_certificate_id: cert_id,
        upload_date: Utc::now(),
        expiration: None, // Would need to parse cert to get actual expiration
    };

    Ok(ServerCertificate {
        server_certificate_metadata: metadata,
        certificate_body,
        certificate_chain,
        tags,
        wami_arn,
        providers: Vec::new(),
    })
}
