//! Signing Certificate Builder

use super::model::*;
use crate::provider::{CloudProvider, ResourceType};
use chrono::Utc;

/// Build a new SigningCertificate resource
pub fn build_signing_certificate(
    user_name: String,
    certificate_body: String,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> SigningCertificate {
    let certificate_id = provider.generate_resource_id(ResourceType::SigningCertificate);
    let wami_arn = provider.generate_wami_arn(
        ResourceType::SigningCertificate,
        account_id,
        "/",
        &certificate_id,
    );

    SigningCertificate {
        user_name,
        certificate_id,
        certificate_body,
        status: CertificateStatus::Active,
        upload_date: Utc::now(),
        wami_arn,
        providers: Vec::new(),
    }
}
