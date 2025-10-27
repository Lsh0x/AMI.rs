//! Assume Role Operations

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
    /// Assumes an IAM role and returns temporary security credentials
    ///
    /// Returns temporary security credentials that you can use to access AWS resources.
    /// These credentials consist of an access key ID, a secret access key, and a security token.
    ///
    /// # Arguments
    ///
    /// * `request` - The assume role request parameters
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryStsClient, sts::AssumeRoleRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut sts_client = MemoryStsClient::new(store);
    ///
    /// let request = AssumeRoleRequest {
    ///     role_arn: "arn:aws:iam::123456789012:role/DataScientist".to_string(),
    ///     role_session_name: "analytics-session".to_string(),
    ///     duration_seconds: Some(3600),
    ///     external_id: None,
    ///     policy: None,
    /// };
    ///
    /// let response = sts_client.assume_role(request).await?;
    /// let result = response.data.unwrap();
    /// println!("Access Key: {}", result.credentials.access_key_id);
    /// println!("Expires: {}", result.credentials.expiration);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn assume_role(
        &mut self,
        request: AssumeRoleRequest,
    ) -> Result<AmiResponse<AssumeRoleResponse>> {
        // 1. Validate request
        request.validate()?;

        // 2. Get duration with default
        let duration = request.duration_seconds.unwrap_or(3600);

        // 3. Generate temporary credentials
        let creds = credentials::build_temporary_credentials(self.cloud_provider(), duration)?;

        // 4. Get account ID
        let account_id = self.account_id().await?;

        // 5. Extract role name from ARN
        let role_name = request
            .role_arn
            .rsplit('/')
            .next()
            .unwrap_or("assumed-role")
            .to_string();

        // 6. Generate WAMI ARN
        let wami_arn = self.cloud_provider().generate_wami_arn(
            ResourceType::StsAssumedRole,
            &account_id,
            "",
            &creds.session_token,
        );

        // 7. Generate provider config
        let provider_config = crate::provider::ProviderConfig {
            provider_name: self.cloud_provider().name().to_string(),
            account_id: account_id.clone(),
            native_arn: self.cloud_provider().generate_resource_identifier(
                ResourceType::StsAssumedRole,
                &account_id,
                &format!("/assumed-role/{}/", role_name),
                &request.role_session_name,
            ),
            synced_at: chrono::Utc::now(),
            tenant_id: None, // STS sessions are not currently tenant-scoped
        };

        // 8. Create session
        let session_obj = session::StsSession {
            session_token: creds.session_token.clone(),
            access_key_id: creds.access_key_id.clone(),
            secret_access_key: creds.secret_access_key.clone(),
            expiration: creds.expiration,
            status: session::SessionStatus::Active,
            assumed_role_arn: Some(request.role_arn.clone()),
            federated_user_name: None,
            principal_arn: None,
            wami_arn,
            providers: vec![provider_config],
            created_at: chrono::Utc::now(),
            last_used: None,
        };

        // 9. Store session
        {
            let store_ref = self.sts_store().await?;
            store_ref.create_session(session_obj).await?;
        }

        // 10. Return response
        Ok(AmiResponse::success(AssumeRoleResponse {
            credentials: creds,
            assumed_role_user: AssumedRoleUser {
                assumed_role_id: uuid::Uuid::new_v4().to_string(),
                arn: request.role_arn,
            },
        }))
    }

    /// Assume role with SAML
    pub async fn assume_role_with_saml(
        &mut self,
        role_arn: String,
        _principal_arn: String,
        _saml_assertion: String,
    ) -> Result<AmiResponse<AssumeRoleResponse>> {
        // In a real implementation, this would validate the SAML assertion
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name: "saml-session".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        self.assume_role(request).await
    }

    /// Assume role with web identity
    pub async fn assume_role_with_web_identity(
        &mut self,
        role_arn: String,
        _web_identity_token: String,
        role_session_name: String,
    ) -> Result<AmiResponse<AssumeRoleResponse>> {
        // In a real implementation, this would validate the web identity token
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name,
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        self.assume_role(request).await
    }

    /// Assume role with client grants
    pub async fn assume_role_with_client_grants(
        &mut self,
        role_arn: String,
        _client_grant_token: String,
    ) -> Result<AmiResponse<AssumeRoleResponse>> {
        // In a real implementation, this would validate the client grant token
        let request = AssumeRoleRequest {
            role_arn,
            role_session_name: "client-grants-session".to_string(),
            duration_seconds: Some(3600),
            external_id: None,
            policy: None,
        };

        self.assume_role(request).await
    }
}
