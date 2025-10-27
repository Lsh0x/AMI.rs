//! MfaDevice Operations

use super::{builder, model::MfaDevice, requests::*};
use crate::error::Result;
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;

impl<S: Store> IamClient<S> {
    /// Enable an MFA device for an IAM user
    pub async fn enable_mfa_device(
        &mut self,
        request: EnableMfaDeviceRequest,
    ) -> Result<AmiResponse<MfaDevice>> {
        let account_id = self.account_id().await?;
        let provider = self.cloud_provider();

        let store = self.iam_store().await?;

        // Validate user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            });
        }

        // Validate authentication codes (in real AWS, these would be verified against the device)
        if request.authentication_code_1.len() != 6 || request.authentication_code_2.len() != 6 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Authentication codes must be 6 digits".to_string(),
            });
        }

        let mfa_device = builder::build_mfa_device(
            request.user_name,
            request.serial_number,
            provider.as_ref(),
            &account_id,
        );

        let created_device = store.create_mfa_device(mfa_device).await?;

        Ok(AmiResponse::success(created_device))
    }

    /// Disable an MFA device for an IAM user
    pub async fn disable_mfa_device(
        &mut self,
        user_name: String,
        serial_number: String,
    ) -> Result<AmiResponse<()>> {
        let store = self.iam_store().await?;

        // Validate MFA device exists and belongs to user
        match store.get_mfa_device(&serial_number).await? {
            Some(device) => {
                if device.user_name != user_name {
                    return Err(crate::error::AmiError::InvalidParameter {
                        message: "MFA device does not belong to the specified user".to_string(),
                    });
                }
            }
            None => {
                return Err(crate::error::AmiError::ResourceNotFound {
                    resource: format!("MFA Device: {}", serial_number),
                });
            }
        }

        store.delete_mfa_device(&serial_number).await?;
        Ok(AmiResponse::success(()))
    }

    /// List all MFA devices for a specific IAM user
    pub async fn list_mfa_devices(
        &mut self,
        request: ListMfaDevicesRequest,
    ) -> Result<AmiResponse<Vec<MfaDevice>>> {
        let store = self.iam_store().await?;

        let devices = store.list_mfa_devices(&request.user_name).await?;
        Ok(AmiResponse::success(devices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::user::CreateUserRequest;
    use crate::iam::IamClient;
    use crate::store::memory::InMemoryStore;

    fn create_test_client() -> IamClient<InMemoryStore> {
        let store = InMemoryStore::new();
        IamClient::new(store)
    }

    async fn create_test_user(client: &mut IamClient<InMemoryStore>, user_name: &str) {
        let request = CreateUserRequest {
            user_name: user_name.to_string(),
            path: Some("/".to_string()),
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(request).await.unwrap();
    }

    #[tokio::test]
    async fn test_enable_mfa_device() {
        let mut client = create_test_client();
        create_test_user(&mut client, "testuser").await;

        let request = EnableMfaDeviceRequest {
            user_name: "testuser".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/testuser".to_string(),
            authentication_code_1: "123456".to_string(),
            authentication_code_2: "789012".to_string(),
        };

        let response = client.enable_mfa_device(request).await.unwrap();
        assert!(response.success);

        let device = response.data.unwrap();
        assert_eq!(device.user_name, "testuser");
        assert_eq!(
            device.serial_number,
            "arn:aws:iam::123456789012:mfa/testuser"
        );
    }

    #[tokio::test]
    async fn test_enable_mfa_device_user_not_found() {
        let mut client = create_test_client();

        let request = EnableMfaDeviceRequest {
            user_name: "nonexistent".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/nonexistent".to_string(),
            authentication_code_1: "123456".to_string(),
            authentication_code_2: "789012".to_string(),
        };

        let result = client.enable_mfa_device(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enable_mfa_device_invalid_auth_code() {
        let mut client = create_test_client();
        create_test_user(&mut client, "testuser").await;

        let request = EnableMfaDeviceRequest {
            user_name: "testuser".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/testuser".to_string(),
            authentication_code_1: "123".to_string(), // Invalid: not 6 digits
            authentication_code_2: "789012".to_string(),
        };

        let result = client.enable_mfa_device(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disable_mfa_device() {
        let mut client = create_test_client();
        create_test_user(&mut client, "testuser").await;

        let enable_request = EnableMfaDeviceRequest {
            user_name: "testuser".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/testuser".to_string(),
            authentication_code_1: "123456".to_string(),
            authentication_code_2: "789012".to_string(),
        };
        client.enable_mfa_device(enable_request).await.unwrap();

        let result = client
            .disable_mfa_device(
                "testuser".to_string(),
                "arn:aws:iam::123456789012:mfa/testuser".to_string(),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_disable_mfa_device_wrong_user() {
        let mut client = create_test_client();
        create_test_user(&mut client, "testuser").await;
        create_test_user(&mut client, "otheruser").await;

        let enable_request = EnableMfaDeviceRequest {
            user_name: "testuser".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/testuser".to_string(),
            authentication_code_1: "123456".to_string(),
            authentication_code_2: "789012".to_string(),
        };
        client.enable_mfa_device(enable_request).await.unwrap();

        // Try to disable with wrong user
        let result = client
            .disable_mfa_device(
                "otheruser".to_string(),
                "arn:aws:iam::123456789012:mfa/testuser".to_string(),
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_mfa_devices() {
        let mut client = create_test_client();
        create_test_user(&mut client, "testuser").await;

        // Enable two MFA devices
        for i in 1..=2 {
            let request = EnableMfaDeviceRequest {
                user_name: "testuser".to_string(),
                serial_number: format!("arn:aws:iam::123456789012:mfa/testuser{}", i),
                authentication_code_1: "123456".to_string(),
                authentication_code_2: "789012".to_string(),
            };
            client.enable_mfa_device(request).await.unwrap();
        }

        let list_request = ListMfaDevicesRequest {
            user_name: "testuser".to_string(),
        };
        let response = client.list_mfa_devices(list_request).await.unwrap();
        let devices = response.data.unwrap();

        assert_eq!(devices.len(), 2);
    }

    #[tokio::test]
    async fn test_list_mfa_devices_empty() {
        let mut client = create_test_client();
        create_test_user(&mut client, "testuser").await;

        let list_request = ListMfaDevicesRequest {
            user_name: "testuser".to_string(),
        };
        let response = client.list_mfa_devices(list_request).await.unwrap();
        let devices = response.data.unwrap();

        assert_eq!(devices.len(), 0);
    }
}
