//! Federation Operations

use super::model::*;
use super::requests::*;
use crate::error::Result;
use crate::provider::ResourceType;
use crate::store::{Store, StsStore};
use crate::sts::{credentials, session, StsClient};
use crate::types::AmiResponse;

impl<S: Store> StsClient<S>
where
    S::StsStore: StsStore,
{
    /// Get federation token
    ///
    /// Returns temporary security credentials for a federated user.
    pub async fn get_federation_token(
        &mut self,
        request: GetFederationTokenRequest,
    ) -> Result<AmiResponse<GetFederationTokenResponse>> {
        // 1. Validate request
        request.validate()?;

        // 2. Get duration with default
        let duration = request.duration_seconds.unwrap_or(3600);

        // 3. Generate temporary credentials
        let creds =
            credentials::build_temporary_credentials(self.store.cloud_provider(), duration)?;

        // 4. Get account ID
        let account_id = self.account_id().await?;

        // 5. Generate WAMI ARN
        let wami_arn = self.store.cloud_provider().generate_wami_arn(
            ResourceType::StsFederatedUser,
            &account_id,
            "",
            &request.name,
        );

        // 6. Generate provider config
        let provider_config = crate::provider::ProviderConfig {
            provider_name: self.store.cloud_provider().name().to_string(),
            account_id: account_id.clone(),
            native_arn: self.store.cloud_provider().generate_resource_identifier(
                ResourceType::StsFederatedUser,
                &account_id,
                "/federated-user/",
                &request.name,
            ),
            synced_at: chrono::Utc::now(),
            tenant_id: None, // STS sessions are not currently tenant-scoped
        };

        // 7. Create session
        let session_obj = session::StsSession {
            session_token: creds.session_token.clone(),
            access_key_id: creds.access_key_id.clone(),
            secret_access_key: creds.secret_access_key.clone(),
            expiration: creds.expiration,
            status: session::SessionStatus::Active,
            assumed_role_arn: None,
            federated_user_name: Some(request.name.clone()),
            principal_arn: None,
            wami_arn: wami_arn.clone(),
            providers: vec![provider_config.clone()],
            created_at: chrono::Utc::now(),
            last_used: None,
        };

        // 8. Store session
        {
            let store_ref = self.sts_store().await?;
            store_ref.create_session(session_obj).await?;
        }

        // 9. Return response
        Ok(AmiResponse::success(GetFederationTokenResponse {
            credentials: creds,
            federated_user: FederatedUser {
                arn: provider_config.native_arn,
                federated_user_id: uuid::Uuid::new_v4().to_string(),
            },
        }))
    }
}
