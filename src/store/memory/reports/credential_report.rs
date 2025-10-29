//! Credential Report Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::CredentialReportStore;
use crate::wami::reports::credential_report::CredentialReport;
use async_trait::async_trait;

#[async_trait]
impl CredentialReportStore for InMemoryWamiStore {
    async fn store_credential_report(&mut self, report: CredentialReport) -> Result<()> {
        self.credential_report = Some(report);
        Ok(())
    }

    async fn get_credential_report(&self) -> Result<Option<CredentialReport>> {
        Ok(self.credential_report.clone())
    }
}
