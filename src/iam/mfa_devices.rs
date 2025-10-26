use crate::error::Result;
use crate::iam::MfaDevice;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;
use serde::{Deserialize, Serialize};

/// Request parameters for enabling an MFA device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableMfaDeviceRequest {
    /// The name of the IAM user for whom the MFA device is being enabled
    pub user_name: String,
    /// The serial number that uniquely identifies the MFA device
    pub serial_number: String,
    /// First authentication code from the device
    pub authentication_code_1: String,
    /// Second authentication code from the device
    pub authentication_code_2: String,
}

/// Request parameters for listing MFA devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMfaDevicesRequest {
    /// The name of the user whose MFA devices you want to list
    pub user_name: String,
}

impl<S: Store> crate::iam::IamClient<S> {
    /// Enable an MFA device for an IAM user
    ///
    /// # Arguments
    ///
    /// * `request` - The request containing user name, serial number, and authentication codes
    ///
    /// # Returns
    ///
    /// Returns the enabled MFA device information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The user does not exist
    /// * The authentication codes are invalid (for real MFA, we just validate format here)
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, EnableMfaDeviceRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Create a user first
    /// let user_request = CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: Some("/".to_string()),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_user(user_request).await?;
    ///
    /// // Enable MFA device
    /// let mfa_request = EnableMfaDeviceRequest {
    ///     user_name: "alice".to_string(),
    ///     serial_number: "arn:aws:iam::123456789012:mfa/alice".to_string(),
    ///     authentication_code_1: "123456".to_string(),
    ///     authentication_code_2: "789012".to_string(),
    /// };
    /// let response = iam_client.enable_mfa_device(mfa_request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn enable_mfa_device(
        &mut self,
        request: EnableMfaDeviceRequest,
    ) -> Result<AmiResponse<MfaDevice>> {
        let store = self.iam_store().await?;

        // Validate user exists
        if store.get_user(&request.user_name).await?.is_none() {
            return Err(crate::error::AmiError::ResourceNotFound {
                resource: format!("User: {}", request.user_name),
            });
        }

        // Validate authentication codes (in real AWS, these would be verified against the device)
        // For this mock implementation, we just check they're 6 digits
        if request.authentication_code_1.len() != 6 || request.authentication_code_2.len() != 6 {
            return Err(crate::error::AmiError::InvalidParameter {
                message: "Authentication codes must be 6 digits".to_string(),
            });
        }

        // Create the MFA device
        let mfa_device = MfaDevice {
            user_name: request.user_name.clone(),
            serial_number: request.serial_number.clone(),
            enable_date: chrono::Utc::now(),
        };

        let created_device = store.create_mfa_device(mfa_device).await?;

        Ok(AmiResponse::success(created_device))
    }

    /// Disable an MFA device for an IAM user
    ///
    /// # Arguments
    ///
    /// * `user_name` - The name of the user whose MFA device is being disabled
    /// * `serial_number` - The serial number of the MFA device to disable
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The MFA device does not exist
    /// * The MFA device does not belong to the specified user
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, EnableMfaDeviceRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Setup: create user and enable MFA
    /// let user_request = CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: Some("/".to_string()),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_user(user_request).await?;
    ///
    /// let mfa_request = EnableMfaDeviceRequest {
    ///     user_name: "alice".to_string(),
    ///     serial_number: "arn:aws:iam::123456789012:mfa/alice".to_string(),
    ///     authentication_code_1: "123456".to_string(),
    ///     authentication_code_2: "789012".to_string(),
    /// };
    /// iam_client.enable_mfa_device(mfa_request).await?;
    ///
    /// // Disable the MFA device
    /// iam_client
    ///     .disable_mfa_device(
    ///         "alice".to_string(),
    ///         "arn:aws:iam::123456789012:mfa/alice".to_string(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `request` - The request containing the user name
    ///
    /// # Returns
    ///
    /// Returns a list of MFA devices for the user.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wami::{MemoryIamClient, CreateUserRequest, EnableMfaDeviceRequest, ListMfaDevicesRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = wami::create_memory_store();
    /// let mut iam_client = MemoryIamClient::new(store);
    ///
    /// // Setup: create user and enable MFA
    /// let user_request = CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: Some("/".to_string()),
    ///     permissions_boundary: None,
    ///     tags: None,
    /// };
    /// iam_client.create_user(user_request).await?;
    ///
    /// let mfa_request = EnableMfaDeviceRequest {
    ///     user_name: "alice".to_string(),
    ///     serial_number: "arn:aws:iam::123456789012:mfa/alice".to_string(),
    ///     authentication_code_1: "123456".to_string(),
    ///     authentication_code_2: "789012".to_string(),
    /// };
    /// iam_client.enable_mfa_device(mfa_request).await?;
    ///
    /// // List MFA devices
    /// let list_request = ListMfaDevicesRequest {
    ///     user_name: "alice".to_string(),
    /// };
    /// let response = iam_client.list_mfa_devices(list_request).await?;
    /// let devices = response.data.unwrap();
    /// println!("Found {} MFA devices", devices.len());
    /// # Ok(())
    /// # }
    /// ```
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
    use crate::iam::{CreateUserRequest, IamClient};
    use crate::store::in_memory::InMemoryStore;

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
