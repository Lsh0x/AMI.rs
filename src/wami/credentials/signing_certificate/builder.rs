//! Signing Certificate Builder

use super::model::*;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use chrono::Utc;
use uuid::Uuid;

/// Build a new SigningCertificate resource with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_signing_certificate(
    user_name: String,
    certificate_body: String,
    context: &WamiContext,
) -> Result<SigningCertificate> {
    let certificate_id = Uuid::new_v4().to_string();

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("signing-certificate", &certificate_id)
        .build()?;

    Ok(SigningCertificate {
        user_name,
        certificate_id,
        certificate_body,
        status: CertificateStatus::Active,
        upload_date: Utc::now(),
        wami_arn,
        providers: Vec::new(),
    })
}
