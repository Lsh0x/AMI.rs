//! Credentials Builder Functions

use super::model::Credentials;
use crate::error::Result;
use crate::provider::CloudProvider;

/// Build temporary credentials with provider validation
///
/// This centralizes credential generation logic to avoid duplication
#[allow(clippy::result_large_err)]
pub fn build_temporary_credentials(
    provider: &dyn CloudProvider,
    duration_seconds: i32,
) -> Result<Credentials> {
    // Validate duration against provider limits
    provider.validate_session_duration(duration_seconds)?;

    // Generate session token
    let session_token = uuid::Uuid::new_v4().to_string();

    // Generate access key ID (AWS format: ASIA + 17 chars)
    let access_key_id = format!(
        "ASIA{}",
        uuid::Uuid::new_v4()
            .to_string()
            .replace('-', "")
            .chars()
            .take(17)
            .collect::<String>()
    );

    // Generate secret access key
    let secret_access_key = uuid::Uuid::new_v4().to_string().replace('-', "");

    // Calculate expiration
    let expiration = chrono::Utc::now() + chrono::Duration::seconds(duration_seconds as i64);

    Ok(Credentials {
        access_key_id,
        secret_access_key,
        session_token,
        expiration,
        // These fields will be filled by the caller with proper context
        arn: String::new(),
        wami_arn: String::new(),
        providers: Vec::new(),
        tenant_id: None,
    })
}
