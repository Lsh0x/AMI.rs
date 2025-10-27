//! Session Token Operations

use super::requests::*;
use crate::error::Result;
use crate::provider::ResourceType;
use crate::store::{Store, StsStore};
use crate::sts::{session, Credentials, StsClient};
use crate::types::AmiResponse;

impl<S: Store> StsClient<S>
where
    S::StsStore: StsStore,
{
    /// Get session token
    ///
    /// Returns temporary security credentials for an IAM user or AWS account root user.
    pub async fn get_session_token(
        &mut self,
        request: Option<GetSessionTokenRequest>,
    ) -> Result<AmiResponse<Credentials>> {
        // 1. Validate request if provided
        if let Some(ref req) = request {
            req.validate()?;
        }

        // 2. Get duration with default
        let duration = request
            .as_ref()
            .and_then(|r| r.duration_seconds)
            .unwrap_or(3600);

        // 3. Generate temporary credentials (session tokens have different limits than assumed roles)
        // Session tokens: 900-129600 seconds, Assumed roles: 3600-43200 seconds
        // So we generate credentials directly without provider validation
        let session_token = uuid::Uuid::new_v4().to_string();
        let access_key_id = format!(
            "ASIA{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(17)
                .collect::<String>()
        );
        let secret_access_key = uuid::Uuid::new_v4().to_string().replace('-', "");
        let expiration = chrono::Utc::now() + chrono::Duration::seconds(duration as i64);

        let creds = crate::sts::Credentials {
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
            session_token: session_token.clone(),
            expiration,
            arn: String::new(),      // Will be set below
            wami_arn: String::new(), // Will be set below
            providers: Vec::new(),   // Will be set below
            tenant_id: None,
        };

        // 4. Get account ID
        let account_id = self.account_id().await?;

        // 5. Generate WAMI ARN
        let wami_arn = self.cloud_provider().generate_wami_arn(
            ResourceType::StsSession,
            &account_id,
            "",
            &creds.session_token,
        );

        // 6. Generate provider config
        let provider_config = crate::provider::ProviderConfig {
            provider_name: self.cloud_provider().name().to_string(),
            account_id: account_id.clone(),
            native_arn: self.cloud_provider().generate_resource_identifier(
                ResourceType::StsSession,
                &account_id,
                "/session/",
                &creds.session_token,
            ),
            synced_at: chrono::Utc::now(),
            tenant_id: None, // STS sessions are not currently tenant-scoped
        };

        // 7. Update creds with ARNs and providers
        let mut creds = creds;
        creds.arn = format!(
            "arn:{}:sts::{}:session-token/{}",
            self.cloud_provider().name(),
            account_id,
            creds.session_token
        );
        creds.wami_arn = wami_arn.clone();
        creds.providers = vec![provider_config.clone()];

        // 8. Create session
        let session_obj = session::StsSession {
            session_token: creds.session_token.clone(),
            access_key_id: creds.access_key_id.clone(),
            secret_access_key: creds.secret_access_key.clone(),
            expiration: creds.expiration,
            status: session::SessionStatus::Active,
            assumed_role_arn: None,
            federated_user_name: None,
            principal_arn: None,
            arn: format!(
                "arn:{}:sts::{}:session-token/{}",
                self.cloud_provider().name(),
                account_id,
                creds.session_token
            ),
            wami_arn,
            providers: vec![provider_config],
            tenant_id: None,
            created_at: chrono::Utc::now(),
            last_used: None,
        };

        // 8. Store session
        {
            let store_ref = self.sts_store().await?;
            store_ref.create_session(session_obj).await?;
        }

        // 9. Return credentials
        Ok(AmiResponse::success(creds))
    }
}
