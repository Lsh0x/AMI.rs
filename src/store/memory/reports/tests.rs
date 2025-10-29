//! Tests for Reports Store Implementation

use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::CredentialReportStore;
use crate::wami::reports::credential_report::CredentialReport;

#[tokio::test]
async fn test_credential_report_store_and_get() {
    let mut store = InMemoryWamiStore::new();

    let report = CredentialReport::new(b"user,status\nalice,active".to_vec());
    let report_time = report.generated_time;

    // Store report
    store.store_credential_report(report.clone()).await.unwrap();

    // Get report
    let retrieved = store.get_credential_report().await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().generated_time, report_time);
}

#[tokio::test]
async fn test_credential_report_get_empty() {
    let store = InMemoryWamiStore::new();

    let result = store.get_credential_report().await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_credential_report_replace() {
    let mut store = InMemoryWamiStore::new();

    let report1 = CredentialReport::new(b"first".to_vec());
    store.store_credential_report(report1).await.unwrap();

    // Store another report (should replace the first)
    let report2 = CredentialReport::new(b"second".to_vec());
    let report2_time = report2.generated_time;
    store.store_credential_report(report2).await.unwrap();

    // Should only have one report (the latest)
    let retrieved = store.get_credential_report().await.unwrap().unwrap();
    assert_eq!(retrieved.generated_time, report2_time);
}

#[tokio::test]
async fn test_credential_report_complete_lifecycle() {
    let mut store = InMemoryWamiStore::new();

    // Start with empty
    assert!(store.get_credential_report().await.unwrap().is_none());

    // Store a report
    let report = CredentialReport::new(b"report1".to_vec());
    let report_time = report.generated_time;
    store.store_credential_report(report).await.unwrap();

    // Verify it exists
    let retrieved = store.get_credential_report().await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().generated_time, report_time);

    // Store a new report
    let new_report = CredentialReport::new(b"report2".to_vec());
    let new_report_time = new_report.generated_time;
    store.store_credential_report(new_report).await.unwrap();

    // Verify the new report replaced the old one
    let final_report = store.get_credential_report().await.unwrap().unwrap();
    assert_eq!(final_report.generated_time, new_report_time);
    assert_ne!(final_report.generated_time, report_time);
}
