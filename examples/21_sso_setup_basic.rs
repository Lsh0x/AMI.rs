//! SSO Setup Basic
//!
//! This example demonstrates:
//! - Creating SSO instance
//! - Defining permission sets
//! - Basic SSO configuration
//!
//! Scenario: Setting up SSO for organization.
//!
//! Run with: `cargo run --example 21_sso_setup_basic`

use std::sync::{Arc, RwLock};
use wami::provider::AwsProvider;
use wami::service::{InstanceService as SsoInstanceService, PermissionSetService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::sso_admin::instance::SsoInstance;
use wami::wami::sso_admin::permission_set::PermissionSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SSO Setup Basic ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    let _provider = Arc::new(AwsProvider::new());
    let _account_id = "123456789012";

    // Create SSO instance
    println!("Step 1: Creating SSO instance...\n");
    let instance_service = SsoInstanceService::new(store.clone());

    let instance = SsoInstance {
        instance_arn: "arn:aws:sso:::instance/ssoins-1234567890abcdef".to_string(),
        identity_store_id: "d-1234567890".to_string(),
        name: Some("MyOrganization SSO".to_string()),
        status: "ACTIVE".to_string(),
        created_date: chrono::Utc::now(),
        wami_arn: "arn:wami:sso-admin:123456789012:instance/myorg-sso".to_string(),
        providers: vec![],
    };

    instance_service.create_instance(instance.clone()).await?;
    println!("✓ Created SSO instance: {}", instance.instance_arn);

    // Create permission sets
    println!("\nStep 2: Creating permission sets...\n");
    let perm_service = PermissionSetService::new(store);

    let admin_perm = PermissionSet {
        permission_set_arn: "arn:aws:sso:::permissionSet/ssoins-xxx/ps-admin".to_string(),
        name: "AdministratorAccess".to_string(),
        description: Some("Full admin access".to_string()),
        session_duration: Some("PT8H".to_string()),
        relay_state: None,
        created_date: chrono::Utc::now(),
        instance_arn: instance.instance_arn.clone(),
        wami_arn: "arn:wami:sso-admin:123456789012:permission-set/admin".to_string(),
        providers: vec![],
    };

    perm_service.create_permission_set(admin_perm).await?;
    println!("✓ Created AdministratorAccess permission set");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- SSO instances centralize identity management");
    println!("- Permission sets define access levels");
    println!("- Use for enterprise SSO integration");

    Ok(())
}
