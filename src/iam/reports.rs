//! IAM Credential and Account Reports
//!
//! This module provides functionality for generating and retrieving IAM reports,
//! including credential reports and account summaries.

use crate::error::{AmiError, Result};
use crate::iam::IamClient;
use crate::store::{IamStore, Store};
use crate::types::AmiResponse;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request to get the credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCredentialReportRequest {}

/// Response from getting the credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCredentialReportResponse {
    /// The credential report in CSV format (base64 encoded)
    #[serde(rename = "Content")]
    pub content: String,

    /// The format of the report (always "text/csv")
    #[serde(rename = "ReportFormat")]
    pub report_format: String,

    /// When the report was generated
    #[serde(rename = "GeneratedTime")]
    pub generated_time: DateTime<Utc>,
}

/// Request to generate a new credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCredentialReportRequest {}

/// Response from generating a credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCredentialReportResponse {
    /// The state of the report generation
    #[serde(rename = "State")]
    pub state: ReportState,

    /// Description of the report state
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// State of credential report generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportState {
    /// Report generation is in progress
    #[serde(rename = "STARTED")]
    Started,
    /// Report generation is complete
    #[serde(rename = "COMPLETE")]
    Complete,
    /// Report generation is in progress
    #[serde(rename = "INPROGRESS")]
    InProgress,
}

/// Request to get account summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountSummaryRequest {}

/// Response from getting account summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountSummaryResponse {
    /// Summary map of resource counts and limits
    #[serde(rename = "SummaryMap")]
    pub summary_map: AccountSummaryMap,
}

/// Account summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountSummaryMap {
    /// Number of users
    #[serde(rename = "Users")]
    pub users: u32,

    /// Quota for users
    #[serde(rename = "UsersQuota")]
    pub users_quota: u32,

    /// Number of groups
    #[serde(rename = "Groups")]
    pub groups: u32,

    /// Quota for groups
    #[serde(rename = "GroupsQuota")]
    pub groups_quota: u32,

    /// Number of roles
    #[serde(rename = "Roles")]
    pub roles: u32,

    /// Quota for roles
    #[serde(rename = "RolesQuota")]
    pub roles_quota: u32,

    /// Number of policies
    #[serde(rename = "Policies")]
    pub policies: u32,

    /// Quota for policies
    #[serde(rename = "PoliciesQuota")]
    pub policies_quota: u32,

    /// Number of MFA devices
    #[serde(rename = "MFADevices")]
    pub mfa_devices: u32,

    /// Number of MFA devices in use
    #[serde(rename = "MFADevicesInUse")]
    pub mfa_devices_in_use: u32,

    /// Number of server certificates
    #[serde(rename = "ServerCertificates")]
    pub server_certificates: u32,

    /// Quota for server certificates
    #[serde(rename = "ServerCertificatesQuota")]
    pub server_certificates_quota: u32,

    /// Number of access keys per user quota
    #[serde(rename = "AccessKeysPerUserQuota")]
    pub access_keys_per_user_quota: u32,

    /// Number of signing certificates per user quota
    #[serde(rename = "SigningCertificatesPerUserQuota")]
    pub signing_certificates_per_user_quota: u32,
}

/// Stored credential report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialReport {
    /// The report content in CSV format
    pub content: String,
    /// When the report was generated
    pub generated_time: DateTime<Utc>,
    /// State of the report
    pub state: ReportState,
}

impl<S: Store> IamClient<S> {
    /// Generate a credential report
    ///
    /// Generates a credential report that lists all IAM users and the status of their
    /// various credentials, including passwords, access keys, and MFA devices.
    ///
    /// # Arguments
    ///
    /// * `_request` - The credential report generation request (currently unused)
    ///
    /// # Returns
    ///
    /// Returns the state of report generation
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyiam::{MemoryIamClient, GenerateCredentialReportRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// let request = GenerateCredentialReportRequest {};
    /// let response = client.generate_credential_report(request).await?;
    ///
    /// assert_eq!(response.data.unwrap().state, rustyiam::ReportState::Complete);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_credential_report(
        &mut self,
        _request: GenerateCredentialReportRequest,
    ) -> Result<AmiResponse<GenerateCredentialReportResponse>> {
        let store = self.iam_store().await?;

        // Get all users
        let (users, _, _) = store.list_users(None, None).await?;

        // Generate CSV content
        let mut csv_lines = vec![
            "user,arn,user_creation_time,password_enabled,password_last_used,password_last_changed,password_next_rotation,mfa_active,access_key_1_active,access_key_1_last_rotated,access_key_1_last_used_date,access_key_2_active,access_key_2_last_rotated,access_key_2_last_used_date".to_string()
        ];

        for user in users {
            // Check for password
            let login_profile = store.get_login_profile(&user.user_name).await?;
            let password_enabled = if login_profile.is_some() {
                "true"
            } else {
                "false"
            };
            let password_last_changed = if let Some(ref profile) = login_profile {
                profile.create_date.to_rfc3339()
            } else {
                "N/A".to_string()
            };

            // Check for MFA devices
            let mfa_devices = store.list_mfa_devices(&user.user_name).await?;
            let mfa_active = if mfa_devices.is_empty() {
                "false"
            } else {
                "true"
            };

            // Check for access keys
            let (access_keys, _, _) = store.list_access_keys(&user.user_name, None).await?;
            let key1_active = access_keys
                .first()
                .map(|k| {
                    if k.status == "Active" {
                        "true"
                    } else {
                        "false"
                    }
                })
                .unwrap_or("false");
            let key1_last_rotated = access_keys
                .first()
                .map(|k| k.create_date.to_rfc3339())
                .unwrap_or_else(|| "N/A".to_string());
            let key1_last_used = "N/A"; // Not tracked yet

            let key2_active = access_keys
                .get(1)
                .map(|k| {
                    if k.status == "Active" {
                        "true"
                    } else {
                        "false"
                    }
                })
                .unwrap_or("false");
            let key2_last_rotated = access_keys
                .get(1)
                .map(|k| k.create_date.to_rfc3339())
                .unwrap_or_else(|| "N/A".to_string());
            let key2_last_used = "N/A"; // Not tracked yet

            // Build CSV line
            let line = format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                user.user_name,
                user.arn,
                user.create_date.to_rfc3339(),
                password_enabled,
                "N/A", // password_last_used (not tracked)
                password_last_changed,
                "N/A", // password_next_rotation (not implemented)
                mfa_active,
                key1_active,
                key1_last_rotated,
                key1_last_used,
                key2_active,
                key2_last_rotated,
                key2_last_used
            );
            csv_lines.push(line);
        }

        let csv_content = csv_lines.join("\n");

        // Store the report
        let report = CredentialReport {
            content: csv_content,
            generated_time: Utc::now(),
            state: ReportState::Complete,
        };

        store.store_credential_report(report).await?;

        Ok(AmiResponse::success(GenerateCredentialReportResponse {
            state: ReportState::Complete,
            description: Some("Report generated successfully".to_string()),
        }))
    }

    /// Get the most recently generated credential report
    ///
    /// Retrieves the credential report that was generated by a previous call to
    /// `generate_credential_report`.
    ///
    /// # Arguments
    ///
    /// * `_request` - The get credential report request (currently unused)
    ///
    /// # Returns
    ///
    /// Returns the credential report in CSV format (base64 encoded)
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyiam::{MemoryIamClient, GenerateCredentialReportRequest, GetCredentialReportRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // First generate a report
    /// client.generate_credential_report(GenerateCredentialReportRequest {}).await?;
    ///
    /// // Then retrieve it
    /// let request = GetCredentialReportRequest {};
    /// let response = client.get_credential_report(request).await?;
    ///
    /// let report = response.data.unwrap();
    /// assert_eq!(report.report_format, "text/csv");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_credential_report(
        &mut self,
        _request: GetCredentialReportRequest,
    ) -> Result<AmiResponse<GetCredentialReportResponse>> {
        let store = self.iam_store().await?;

        let report = store.get_credential_report().await?;

        match report {
            Some(report) => {
                if report.state != ReportState::Complete {
                    return Err(AmiError::InvalidParameter {
                        message: "Report generation is not complete yet".to_string(),
                    });
                }

                // Base64 encode the content
                let encoded_content =
                    base64::engine::general_purpose::STANDARD.encode(report.content.as_bytes());

                Ok(AmiResponse::success(GetCredentialReportResponse {
                    content: encoded_content,
                    report_format: "text/csv".to_string(),
                    generated_time: report.generated_time,
                }))
            }
            None => Err(AmiError::ResourceNotFound {
                resource: "Credential report".to_string(),
            }),
        }
    }

    /// Get account summary
    ///
    /// Retrieves information about IAM entity usage and IAM quotas in the account.
    ///
    /// # Arguments
    ///
    /// * `_request` - The get account summary request (currently unused)
    ///
    /// # Returns
    ///
    /// Returns statistics about IAM resources and quotas
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustyiam::{MemoryIamClient, GetAccountSummaryRequest, CreateUserRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = rustyiam::create_memory_store();
    /// let mut client = MemoryIamClient::new(store);
    ///
    /// // Create a user
    /// client.create_user(CreateUserRequest {
    ///     user_name: "alice".to_string(),
    ///     path: None,
    ///     permissions_boundary: None,
    ///     tags: None,
    /// }).await?;
    ///
    /// // Get account summary
    /// let request = GetAccountSummaryRequest {};
    /// let response = client.get_account_summary(request).await?;
    ///
    /// let summary = response.data.unwrap().summary_map;
    /// assert_eq!(summary.users, 1);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_account_summary(
        &mut self,
        _request: GetAccountSummaryRequest,
    ) -> Result<AmiResponse<GetAccountSummaryResponse>> {
        let store = self.iam_store().await?;

        // Count all resources
        let (users, _, _) = store.list_users(None, None).await?;
        let (groups, _, _) = store.list_groups(None, None).await?;
        let (roles, _, _) = store.list_roles(None, None).await?;
        let (policies, _, _) = store.list_policies(None, None).await?;

        // Count MFA devices across all users
        let mut total_mfa_devices = 0;
        for user in &users {
            let mfa_devices = store.list_mfa_devices(&user.user_name).await?;
            total_mfa_devices += mfa_devices.len() as u32;
        }

        // AWS default quotas (these can be made configurable in the future)
        let summary_map = AccountSummaryMap {
            users: users.len() as u32,
            users_quota: 5000,
            groups: groups.len() as u32,
            groups_quota: 300,
            roles: roles.len() as u32,
            roles_quota: 1000,
            policies: policies.len() as u32,
            policies_quota: 1500,
            mfa_devices: total_mfa_devices,
            mfa_devices_in_use: total_mfa_devices,
            server_certificates: 0, // Not implemented yet
            server_certificates_quota: 20,
            access_keys_per_user_quota: 2,
            signing_certificates_per_user_quota: 2,
        };

        Ok(AmiResponse::success(GetAccountSummaryResponse {
            summary_map,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::mfa_devices::EnableMfaDeviceRequest;
    use crate::iam::users::CreateUserRequest;

    #[tokio::test]
    async fn test_generate_and_get_credential_report() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create a test user
        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        // Generate credential report
        let gen_request = GenerateCredentialReportRequest {};
        let gen_response = client
            .generate_credential_report(gen_request)
            .await
            .unwrap();
        assert!(gen_response.success);
        assert_eq!(gen_response.data.unwrap().state, ReportState::Complete);

        // Get credential report
        let get_request = GetCredentialReportRequest {};
        let get_response = client.get_credential_report(get_request).await.unwrap();
        assert!(get_response.success);

        let report = get_response.data.unwrap();
        assert_eq!(report.report_format, "text/csv");

        // Decode and check content
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(report.content.as_bytes())
            .unwrap();
        let content = String::from_utf8(decoded).unwrap();
        assert!(content.contains("alice"));
        assert!(content.contains("user,arn,user_creation_time"));
    }

    #[tokio::test]
    async fn test_get_credential_report_not_found() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        let request = GetCredentialReportRequest {};
        let result = client.get_credential_report(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_account_summary() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create some test resources
        let user_request = CreateUserRequest {
            user_name: "alice".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        // Add MFA device
        let mfa_request = EnableMfaDeviceRequest {
            user_name: "alice".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/alice".to_string(),
            authentication_code_1: "123456".to_string(),
            authentication_code_2: "654321".to_string(),
        };
        client.enable_mfa_device(mfa_request).await.unwrap();

        // Get account summary
        let request = GetAccountSummaryRequest {};
        let response = client.get_account_summary(request).await.unwrap();
        assert!(response.success);

        let summary = response.data.unwrap().summary_map;
        assert_eq!(summary.users, 1);
        assert_eq!(summary.mfa_devices, 1);
        assert_eq!(summary.users_quota, 5000);
    }

    #[tokio::test]
    async fn test_credential_report_with_mfa() {
        let store = crate::store::in_memory::InMemoryStore::new();
        let mut client = IamClient::new(store);

        // Create user with MFA
        let user_request = CreateUserRequest {
            user_name: "bob".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        client.create_user(user_request).await.unwrap();

        let mfa_request = EnableMfaDeviceRequest {
            user_name: "bob".to_string(),
            serial_number: "arn:aws:iam::123456789012:mfa/bob".to_string(),
            authentication_code_1: "123456".to_string(),
            authentication_code_2: "654321".to_string(),
        };
        client.enable_mfa_device(mfa_request).await.unwrap();

        // Generate and get report
        client
            .generate_credential_report(GenerateCredentialReportRequest {})
            .await
            .unwrap();

        let get_request = GetCredentialReportRequest {};
        let response = client.get_credential_report(get_request).await.unwrap();

        let report = response.data.unwrap();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(report.content.as_bytes())
            .unwrap();
        let content = String::from_utf8(decoded).unwrap();

        // Check that MFA is marked as active
        assert!(content.contains("bob"));
        // The line should have "true" in the mfa_active column (8th column)
        let bob_line = content.lines().find(|l| l.starts_with("bob")).unwrap();
        let fields: Vec<&str> = bob_line.split(',').collect();
        assert_eq!(fields[7], "true"); // mfa_active column
    }
}
