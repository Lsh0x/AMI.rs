//! Credential Report Service
//!
//! Orchestrates credential report generation and account summary operations.

use crate::error::{AmiError, Result};
use crate::provider::{AwsProvider, CloudProvider};
use crate::store::traits::{
    AccessKeyStore, CredentialReportStore, GroupStore, LoginProfileStore, MfaDeviceStore,
    PolicyStore, RoleStore, ServerCertificateStore, UserStore,
};
use crate::wami::reports::credential_report::{
    AccountSummaryMap, CredentialReport, CredentialReportStatus, GenerateCredentialReportRequest,
    GenerateCredentialReportResponse, GetAccountSummaryRequest, GetAccountSummaryResponse,
    GetCredentialReportRequest, GetCredentialReportResponse,
};
use std::sync::{Arc, RwLock};

/// Service for generating credential reports and account summaries
///
/// Provides high-level operations for IAM reporting and auditing.
pub struct CredentialReportService<S> {
    store: Arc<RwLock<S>>,
    #[allow(dead_code)]
    provider: Arc<dyn CloudProvider>,
    account_id: String,
}

impl<S> CredentialReportService<S>
where
    S: CredentialReportStore
        + UserStore
        + GroupStore
        + RoleStore
        + PolicyStore
        + MfaDeviceStore
        + AccessKeyStore
        + LoginProfileStore
        + ServerCertificateStore,
{
    /// Create a new CredentialReportService with default AWS provider
    pub fn new(store: Arc<RwLock<S>>, account_id: String) -> Self {
        Self {
            store,
            provider: Arc::new(AwsProvider::new()),
            account_id,
        }
    }

    /// Returns a new service instance with different provider
    pub fn with_provider(&self, provider: Arc<dyn CloudProvider>) -> Self {
        Self {
            store: self.store.clone(),
            provider,
            account_id: self.account_id.clone(),
        }
    }

    /// Generate a new credential report
    ///
    /// Creates a CSV report of all IAM users and their credential status.
    pub async fn generate_credential_report(
        &self,
        _request: GenerateCredentialReportRequest,
    ) -> Result<GenerateCredentialReportResponse> {
        // Fetch all users
        let (users, _, _) = self.store.read().unwrap().list_users(None, None).await?;

        // Generate CSV content
        let mut csv_content = String::from(
            "user,arn,user_creation_time,password_enabled,password_last_used,password_last_changed,\
             mfa_active,access_key_1_active,access_key_1_last_rotated,access_key_2_active,\
             access_key_2_last_rotated\n",
        );

        for user in users {
            // Fetch user's credentials
            let mfa_devices = self
                .store
                .read()
                .unwrap()
                .list_mfa_devices(&user.user_name)
                .await?;

            let access_keys = self
                .store
                .read()
                .unwrap()
                .list_access_keys(&user.user_name, None)
                .await?
                .0;

            let has_login_profile = self
                .store
                .read()
                .unwrap()
                .get_login_profile(&user.user_name)
                .await?
                .is_some();

            // Add row for this user
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                user.user_name,
                user.arn,
                user.create_date.to_rfc3339(),
                has_login_profile,
                user.password_last_used
                    .map_or("N/A".to_string(), |d| d.to_rfc3339()),
                "N/A", // password_last_changed not tracked yet
                !mfa_devices.is_empty(),
                access_keys
                    .first()
                    .map_or("false", |k| if k.status == "Active" {
                        "true"
                    } else {
                        "false"
                    }),
                access_keys
                    .first()
                    .map_or("N/A".to_string(), |k| k.create_date.to_rfc3339()),
                access_keys
                    .get(1)
                    .map_or("false", |k| if k.status == "Active" {
                        "true"
                    } else {
                        "false"
                    }),
                access_keys
                    .get(1)
                    .map_or("N/A".to_string(), |k| k.create_date.to_rfc3339()),
            ));
        }

        // Create and store report
        let report = CredentialReport::new(csv_content.into_bytes());

        self.store
            .write()
            .unwrap()
            .store_credential_report(report)
            .await?;

        Ok(GenerateCredentialReportResponse {
            state: CredentialReportStatus::Complete,
            description: Some("Report generated successfully".to_string()),
        })
    }

    /// Get the most recent credential report
    pub async fn get_credential_report(
        &self,
        _request: GetCredentialReportRequest,
    ) -> Result<GetCredentialReportResponse> {
        let report = self
            .store
            .read()
            .unwrap()
            .get_credential_report()
            .await?
            .ok_or_else(|| AmiError::ResourceNotFound {
                resource: "No credential report has been generated yet".to_string(),
            })?;

        // Encode content as base64
        let content_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &report.report_content,
        );

        Ok(GetCredentialReportResponse {
            content: content_base64,
            report_format: report.report_format,
            generated_time: report.generated_time,
        })
    }

    /// Get account summary with resource counts and quotas
    pub async fn get_account_summary(
        &self,
        _request: GetAccountSummaryRequest,
    ) -> Result<GetAccountSummaryResponse> {
        let store_read = self.store.read().unwrap();

        // Count users
        let (users, _, _) = store_read.list_users(None, None).await?;
        let users_count = users.len() as u32;

        // Count groups
        let (groups, _, _) = store_read.list_groups(None, None).await?;
        let groups_count = groups.len() as u32;

        // Count roles
        let (roles, _, _) = store_read.list_roles(None, None).await?;
        let roles_count = roles.len() as u32;

        // Count policies
        let (policies, _, _) = store_read.list_policies(None, None).await?;
        let policies_count = policies.len() as u32;

        // Count MFA devices (across all users)
        let mut mfa_count = 0;
        for user in &users {
            let devices = store_read.list_mfa_devices(&user.user_name).await?;
            mfa_count += devices.len() as u32;
        }

        // Count server certificates
        let (certs, _, _) = store_read.list_server_certificates(None, None).await?;
        let certs_count = certs.len() as u32;

        drop(store_read);

        // Build summary map with AWS default quotas
        let summary = AccountSummaryMap {
            users: users_count,
            users_quota: 5000, // AWS default
            groups: groups_count,
            groups_quota: 300, // AWS default
            roles: roles_count,
            roles_quota: 1000, // AWS default
            policies: policies_count,
            policies_quota: 1500, // AWS default (customer managed)
            mfa_devices: mfa_count,
            mfa_devices_in_use: mfa_count, // Assume all are in use
            server_certificates: certs_count,
            server_certificates_quota: 20,          // AWS default
            access_keys_per_user_quota: 2,          // AWS default
            signing_certificates_per_user_quota: 2, // AWS default
        };

        Ok(GetAccountSummaryResponse {
            summary_map: summary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::{TenantPath, WamiArn};
    use crate::context::WamiContext;
    use crate::store::memory::InMemoryWamiStore;
    use crate::wami::identity::user::builder::build_user;

    fn setup_service() -> CredentialReportService<InMemoryWamiStore> {
        let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
        CredentialReportService::new(store, "123456789012".to_string())
    }

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:.*:12345678:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_generate_credential_report() {
        let service = setup_service();
        let context = test_context();

        // Create some test users
        for i in 0..3 {
            let user = build_user(format!("user{}", i), Some("/".to_string()), &context).unwrap();
            service
                .store
                .write()
                .unwrap()
                .create_user(user)
                .await
                .unwrap();
        }

        let request = GenerateCredentialReportRequest {};
        let response = service.generate_credential_report(request).await.unwrap();

        assert_eq!(response.state, CredentialReportStatus::Complete);
        assert!(response.description.is_some());
    }

    #[tokio::test]
    async fn test_get_credential_report() {
        let service = setup_service();
        let context = test_context();

        // Create a user
        let user = build_user("alice".to_string(), Some("/".to_string()), &context).unwrap();
        service
            .store
            .write()
            .unwrap()
            .create_user(user)
            .await
            .unwrap();

        // Generate report
        let gen_request = GenerateCredentialReportRequest {};
        service
            .generate_credential_report(gen_request)
            .await
            .unwrap();

        // Get report
        let get_request = GetCredentialReportRequest {};
        let response = service.get_credential_report(get_request).await.unwrap();

        assert_eq!(response.report_format, "text/csv");
        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_get_credential_report_not_generated() {
        let service = setup_service();

        let request = GetCredentialReportRequest {};
        let result = service.get_credential_report(request).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(AmiError::ResourceNotFound { .. })));
    }

    #[tokio::test]
    async fn test_get_account_summary_empty() {
        let service = setup_service();

        let request = GetAccountSummaryRequest {};
        let response = service.get_account_summary(request).await.unwrap();

        assert_eq!(response.summary_map.users, 0);
        assert_eq!(response.summary_map.groups, 0);
        assert_eq!(response.summary_map.roles, 0);
        assert_eq!(response.summary_map.policies, 0);
        assert_eq!(response.summary_map.mfa_devices, 0);
    }

    #[tokio::test]
    async fn test_get_account_summary_with_resources() {
        let service = setup_service();
        let context = test_context();

        // Create users
        for i in 0..5 {
            let user = build_user(format!("user{}", i), Some("/".to_string()), &context).unwrap();
            service
                .store
                .write()
                .unwrap()
                .create_user(user)
                .await
                .unwrap();
        }

        let request = GetAccountSummaryRequest {};
        let response = service.get_account_summary(request).await.unwrap();

        assert_eq!(response.summary_map.users, 5);
        assert_eq!(response.summary_map.users_quota, 5000); // AWS default quota
    }
}
