//! Identity Operations

use super::model::*;
use crate::error::Result;
use crate::store::{Store, StsStore};
use crate::sts::StsClient;
use crate::types::AmiResponse;

impl<S: Store> StsClient<S>
where
    S::StsStore: StsStore,
{
    /// Returns details about the IAM identity whose credentials are used to call this operation
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::MemoryStsClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut sts_client = MemoryStsClient::new(store);
    ///
    /// let response = sts_client.get_caller_identity().await?;
    /// let identity = response.data.unwrap();
    ///
    /// println!("User ID: {}", identity.user_id);
    /// println!("Account: {}", identity.account);
    /// println!("ARN: {}", identity.arn);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_caller_identity(&mut self) -> Result<AmiResponse<CallerIdentity>> {
        let store = self.sts_store().await?;
        let account_id = store.account_id();

        // Try to get existing identity, or create a default one
        let identity_arn = format!("arn:aws:iam::{}:user/example-user", account_id);
        let wami_arn = format!("arn:wami:iam::{}:user/example-user", account_id);

        let identity = store
            .get_identity(&identity_arn)
            .await?
            .unwrap_or_else(|| CallerIdentity {
                user_id: "AIDACKCEVSQ6C2EXAMPLE".to_string(),
                account: account_id.to_string(),
                arn: identity_arn,
                wami_arn,
                providers: Vec::new(),
            });

        Ok(AmiResponse::success(identity))
    }

    /// Get access key info
    pub async fn get_access_key_info(
        &mut self,
        _access_key_id: String,
    ) -> Result<AmiResponse<String>> {
        let store = self.sts_store().await?;
        let account_id = store.account_id();
        Ok(AmiResponse::success(account_id.to_string()))
    }

    /// Decode authorization message
    pub async fn decode_authorization_message(
        &self,
        encoded_message: String,
    ) -> Result<AmiResponse<String>> {
        // In a real implementation, this would decode the authorization message
        // For now, return a placeholder decoded message
        let decoded = format!("Decoded message for: {}", encoded_message);
        Ok(AmiResponse::success(decoded))
    }
}
