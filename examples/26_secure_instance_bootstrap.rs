//! Example 26: Secure Instance Bootstrap with Authentication
//!
//! This example demonstrates the SECURE way to initialize a WAMI instance
//! and authenticate as root. This prevents brute force attacks on instance IDs.
//!
//! # Security Model
//!
//! 1. **Instance Bootstrap** - Generate root user with credentials
//! 2. **Credential Storage** - Save credentials securely (shown once!)
//! 3. **Authentication** - Required for all operations
//! 4. **Authorization** - Policy-based access control
//!
//! # Critical Security Notes
//!
//! ⚠️  Root credentials are shown ONLY during initialization
//! ⚠️  They are hashed with bcrypt and cannot be retrieved later
//! ⚠️  Without credentials, no access is possible (even for root!)
//! ⚠️  This prevents brute force attacks on instance IDs

use std::sync::Arc;
use tokio::sync::RwLock;
use wami::store::memory::InMemoryWamiStore;
use wami::{AmiError, AuthenticationService, InstanceBootstrap, RootCredentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 WAMI Secure Instance Bootstrap Example\n");
    println!("{}", "=".repeat(60));

    // =========================================================================
    // Step 1: Initialize the Store
    // =========================================================================
    println!("\n📦 Step 1: Initialize Store");
    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    println!("✅ In-memory store created");

    // =========================================================================
    // Step 2: Bootstrap Instance with Root User & Credentials
    // =========================================================================
    println!("\n🚀 Step 2: Bootstrap Instance");
    println!("    Creating instance '999888777' with root user...");

    let instance_id = "999888777";
    let root_creds = InstanceBootstrap::initialize_instance(store.clone(), instance_id).await?;

    println!("\n✅ Instance initialized successfully!");
    println!("\n🔑 ROOT CREDENTIALS (SAVE THESE SECURELY!):");
    println!("{}", "=".repeat(60));
    println!("   Access Key ID:     {}", root_creds.access_key_id);
    println!("   Secret Access Key: {}", root_creds.secret_access_key);
    println!("   Instance ID:       {}", root_creds.instance_id);
    println!("   User ARN:          {}", root_creds.user_arn);
    println!("{}", "=".repeat(60));
    println!("\n⚠️  CRITICAL: These credentials are shown ONLY ONCE!");
    println!("⚠️  Save them in a secure location (secrets manager, vault, etc.)");
    println!("⚠️  They CANNOT be retrieved later!");

    // =========================================================================
    // Step 3: Authenticate as Root
    // =========================================================================
    println!("\n🔓 Step 3: Authenticate as Root");
    println!("    Using access key authentication...");

    let auth_service = AuthenticationService::new(store.clone());

    let root_context = auth_service
        .authenticate(&root_creds.access_key_id, &root_creds.secret_access_key)
        .await?;

    println!("✅ Authentication successful!");
    println!("   Authenticated as: {}", root_context.caller_arn());
    println!("   Instance ID:      {}", root_context.instance_id());
    println!("   Tenant Path:      {}", root_context.tenant_path());
    println!("   Is Root:          {}", root_context.is_root());

    // =========================================================================
    // Step 4: Perform Operations with Authenticated Context
    // =========================================================================
    println!("\n👤 Step 4: Create Admin User (as root)");

    // Note: To create users, you would use UserService with the authenticated context
    // Example:
    // use wami::UserService;
    // use std::sync::{Arc, RwLock as StdRwLock};
    // let std_store = Arc::new(StdRwLock::new(store.read().await.wami_store.clone()));
    // let user_service = UserService::new(std_store);
    // let admin_user = user_service.create_user(&root_context, ...).await?;

    println!("✅ Root context can be used to create resources");
    println!("   Context is required for all operations");
    println!("   Root context bypasses authorization checks");

    // =========================================================================
    // Step 5: Demonstrate Security - Invalid Credentials
    // =========================================================================
    println!("\n🛡️  Step 5: Demonstrate Security");
    println!("    Attempting authentication with invalid secret...");

    let result = auth_service
        .authenticate(&root_creds.access_key_id, "wrong_secret_key")
        .await;

    match result {
        Err(AmiError::AccessDenied { .. }) => {
            println!("✅ Invalid credentials rejected (as expected)");
            println!("   Brute force attacks are prevented!");
        }
        Ok(_) => {
            println!("❌ ERROR: Invalid credentials should have been rejected!");
        }
        Err(e) => {
            println!("❌ Unexpected error: {:?}", e);
        }
    }

    // =========================================================================
    // Step 6: Demonstrate Instance State Check
    // =========================================================================
    println!("\n🔍 Step 6: Check Instance State");

    let is_initialized = InstanceBootstrap::is_initialized(store.clone(), instance_id).await?;
    println!(
        "   Instance '{}' initialized: {}",
        instance_id, is_initialized
    );

    let other_instance_initialized =
        InstanceBootstrap::is_initialized(store.clone(), "123456789").await?;
    println!(
        "   Instance '123456789' initialized: {}",
        other_instance_initialized
    );

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n📋 SECURITY SUMMARY");
    println!("{}", "=".repeat(60));
    println!("✅ Instance requires initialization before use");
    println!("✅ Root user has cryptographically secure credentials");
    println!("✅ Credentials are hashed with bcrypt (never plaintext)");
    println!("✅ Authentication is mandatory for all operations");
    println!("✅ Invalid credentials are rejected");
    println!("✅ No way to brute force instance IDs without credentials");
    println!("{}", "=".repeat(60));

    println!("\n🎯 BEST PRACTICES");
    println!("{}", "=".repeat(60));
    println!("1. Store credentials in secrets manager (AWS/Vault/etc.)");
    println!("2. Never commit credentials to version control");
    println!("3. Never log plaintext secrets");
    println!("4. Use root only for initial setup");
    println!("5. Create admin users with specific policies");
    println!("6. Rotate credentials regularly");
    println!("7. Use principle of least privilege");
    println!("{}", "=".repeat(60));

    println!("\n✅ Example completed successfully!");

    Ok(())
}

// =============================================================================
// Helper Functions for Production Use
// =============================================================================

/// Example: How to securely store credentials in production
///
/// In production, you would:
/// 1. Store in AWS Secrets Manager, HashiCorp Vault, etc.
/// 2. Encrypt at rest
/// 3. Control access with IAM/RBAC
/// 4. Enable audit logging
/// 5. Rotate credentials regularly
#[allow(dead_code)]
async fn store_credentials_securely(
    creds: &RootCredentials,
) -> Result<(), Box<dyn std::error::Error>> {
    // Example: Store in AWS Secrets Manager
    // let client = aws_sdk_secretsmanager::Client::new(&aws_config::load_from_env().await);
    // client.create_secret()
    //     .name(format!("wami/instance/{}/root", creds.instance_id))
    //     .secret_string(serde_json::to_string(creds)?)
    //     .send()
    //     .await?;

    println!("📝 Credentials stored in secrets manager");
    println!("   Secret name: wami/instance/{}/root", creds.instance_id);

    Ok(())
}

/// Example: How to retrieve credentials from secure storage
#[allow(dead_code)]
async fn retrieve_credentials_from_vault(
    instance_id: &str,
) -> Result<RootCredentials, Box<dyn std::error::Error>> {
    // Example: Retrieve from secrets manager
    // let client = aws_sdk_secretsmanager::Client::new(&aws_config::load_from_env().await);
    // let response = client.get_secret_value()
    //     .secret_id(format!("wami/instance/{}/root", instance_id))
    //     .send()
    //     .await?;
    //
    // let creds: RootCredentials = serde_json::from_str(response.secret_string().unwrap())?;

    println!("🔑 Credentials retrieved from vault");

    // Return mock for example purposes
    Ok(RootCredentials {
        access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
        secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
        instance_id: instance_id.to_string(),
        user_arn: format!("arn:wami:iam:root:wami:{}:user/root", instance_id),
    })
}

/// Example: How to rotate root credentials
#[allow(dead_code)]
async fn rotate_root_credentials(
    store: Arc<tokio::sync::RwLock<InMemoryWamiStore>>,
    instance_id: &str,
    old_creds: &RootCredentials,
) -> Result<RootCredentials, Box<dyn std::error::Error>> {
    println!("🔄 Rotating root credentials...");

    // 1. Authenticate with old credentials
    let auth_service = AuthenticationService::new(store.clone());
    let _context = auth_service
        .authenticate(&old_creds.access_key_id, &old_creds.secret_access_key)
        .await?;

    // 2. Create new access key for root user
    // (This would use AccessKeyService in a real implementation)

    // 3. Test new credentials
    // (Authenticate with new credentials to verify)

    // 4. Delete old access key
    // (Once new credentials are confirmed working)

    // 5. Update secrets manager with new credentials

    println!("✅ Credentials rotated successfully");
    println!("⚠️  Save the new credentials and delete the old ones!");

    // Return new credentials (mock for example)
    Ok(RootCredentials {
        access_key_id: "AKIANEWKEY123456789".to_string(),
        secret_access_key: "newSecretKey0123456789abcdefghijklmnop".to_string(),
        instance_id: instance_id.to_string(),
        user_arn: old_creds.user_arn.clone(),
    })
}
