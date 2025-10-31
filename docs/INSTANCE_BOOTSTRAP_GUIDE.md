# Instance Bootstrap & Authentication Guide

**Version:** 2.0.0+  
**Date:** October 30, 2025

---

## Overview

This guide covers how to securely initialize WAMI instances and authenticate users. Starting from version 2.0.0, **all access requires authentication** to prevent brute force attacks.

---

## Table of Contents

1. [Security Model](#security-model)
2. [Instance Bootstrap](#instance-bootstrap)
3. [Authentication](#authentication)
4. [Credential Management](#credential-management)
5. [Best Practices](#best-practices)
6. [Common Patterns](#common-patterns)
7. [Troubleshooting](#troubleshooting)

---

## Security Model

### Why Authentication is Required

**Problem:** Without requiring credentials, an attacker could brute force instance IDs:

```rust
// ❌ VULNERABLE (old approach)
for instance_id in 0..999999999 {
    let context = create_context_somehow(&instance_id.to_string());
    // Unauthorized access!
}
```

**Solution:** All access requires valid credentials:

```rust
// ✅ SECURE (new approach)
let context = auth_service
    .authenticate(access_key_id, secret_key)
    .await?;
// Must have valid credentials!
```

### Security Guarantees

✅ **No unauthorized access** - Credentials required  
✅ **No brute force** - Cannot guess instance IDs  
✅ **Credential security** - Bcrypt hashed secrets  
✅ **Timing attack resistance** - Constant-time comparison  
✅ **Audit trail** - All access via authentication  
✅ **Secret irretrievability** - Plaintext never stored  

---

## Instance Bootstrap

### Initializing a New Instance

```rust
use wami::{InstanceBootstrap, InMemoryStore};
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create store
    let store = Arc::new(RwLock::new(InMemoryStore::new()));
    
    // 2. Initialize instance
    let creds = InstanceBootstrap::initialize_instance(
        store.clone(),
        "999888777",  // instance_id
    ).await?;
    
    // 3. CRITICAL: Save credentials securely!
    println!("Access Key: {}", creds.access_key_id);
    println!("Secret Key: {}", creds.secret_access_key);
    // These are shown ONLY ONCE!
    
    save_to_secrets_manager(&creds)?;
    
    Ok(())
}
```

### What Happens During Bootstrap?

1. **Creates root user** with ARN: `arn:wami:iam:root:wami:{instance_id}:user/root`
2. **Generates access key ID**: Format `AKIA + 16 chars` (AWS-compatible)
3. **Generates secret key**: 40 character secure random string
4. **Hashes secret**: Bcrypt with cost factor 12
5. **Stores credentials**: Hashed secret only (never plaintext)
6. **Returns credentials**: Plaintext secret shown ONCE

### Checking if Instance is Initialized

```rust
let is_initialized = InstanceBootstrap::is_initialized(
    store.clone(),
    "999888777",
).await?;

if !is_initialized {
    // Initialize the instance
    let creds = InstanceBootstrap::initialize_instance(
        store.clone(),
        "999888777",
    ).await?;
}
```

---

## Authentication

### Authenticating as Root

```rust
use wami::AuthenticationService;

let auth_service = AuthenticationService::new(store.clone());

// Retrieve credentials from secure storage
let creds = retrieve_from_secrets_manager("999888777")?;

// Authenticate
let context = auth_service
    .authenticate(&creds.access_key_id, &creds.secret_access_key)
    .await?;

// Verify
assert!(context.is_root());
assert_eq!(context.instance_id(), "999888777");
```

### Authenticating as Regular User

```rust
// Same process - all users authenticate the same way
let user_context = auth_service
    .authenticate(&user_access_key_id, &user_secret_key)
    .await?;

// Regular users are NOT root
assert!(!user_context.is_root());

// They have their own ARN
println!("Logged in as: {}", user_context.caller_arn());
```

### Using Authenticated Context

```rust
use wami::UserService;

let user_service = UserService::new(store.clone());

// Context is required for all operations
let user = user_service.create_user(&context, request).await?;
```

---

## Credential Management

### Credential Structure

#### Access Key ID
- **Format:** `AKIA` + 16 uppercase alphanumeric
- **Example:** `AKIAIOSFODNN7EXAMPLE`
- **Storage:** Public, can be logged
- **Length:** 20 characters

#### Secret Access Key
- **Format:** 40 base64-like characters
- **Example:** `wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY`
- **Storage:** NEVER log or store plaintext!
- **Length:** 40 characters
- **Hashing:** Bcrypt (cost 12) before storage

### Storing Credentials Securely

#### Option 1: AWS Secrets Manager

```rust
use aws_sdk_secretsmanager::Client;

async fn store_credentials(
    creds: &RootCredentials,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    client.create_secret()
        .name(format!("wami/instance/{}/root", creds.instance_id))
        .secret_string(serde_json::to_string(creds)?)
        .send()
        .await?;
    
    Ok(())
}

async fn retrieve_credentials(
    instance_id: &str,
) -> Result<RootCredentials, Box<dyn std::error::Error>> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    let response = client.get_secret_value()
        .secret_id(format!("wami/instance/{}/root", instance_id))
        .send()
        .await?;
    
    Ok(serde_json::from_str(response.secret_string().unwrap())?)
}
```

#### Option 2: HashiCorp Vault

```rust
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;

async fn store_in_vault(
    creds: &RootCredentials,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address("https://vault.example.com")
            .token(std::env::var("VAULT_TOKEN")?)
            .build()?,
    )?;
    
    kv2::set(
        &client,
        "secret",
        &format!("wami/instance/{}/root", creds.instance_id),
        &serde_json::to_value(creds)?,
    ).await?;
    
    Ok(())
}
```

#### Option 3: Environment Variables (Development Only!)

```bash
# ⚠️ FOR DEVELOPMENT ONLY - NOT FOR PRODUCTION!
export WAMI_ROOT_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"
export WAMI_ROOT_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
```

```rust
// Load from environment (development only)
let access_key_id = std::env::var("WAMI_ROOT_ACCESS_KEY_ID")?;
let secret_key = std::env::var("WAMI_ROOT_SECRET_ACCESS_KEY")?;
```

### Rotating Credentials

```rust
async fn rotate_credentials(
    store: Arc<RwLock<InMemoryStore>>,
    instance_id: &str,
) -> Result<RootCredentials, Box<dyn std::error::Error>> {
    // 1. Retrieve old credentials
    let old_creds = retrieve_from_secrets_manager(instance_id)?;
    
    // 2. Authenticate with old credentials
    let auth = AuthenticationService::new(store.clone());
    let context = auth
        .authenticate(&old_creds.access_key_id, &old_creds.secret_access_key)
        .await?;
    
    // 3. Create new access key (using AccessKeyService)
    // let new_key = access_key_service
    //     .create_access_key(&context, request)
    //     .await?;
    
    // 4. Test new credentials
    // let test_context = auth
    //     .authenticate(&new_key.access_key_id, &new_key.secret)
    //     .await?;
    
    // 5. Delete old access key
    // access_key_service
    //     .delete_access_key(&context, &old_creds.access_key_id)
    //     .await?;
    
    // 6. Update secrets manager
    // update_secrets_manager(&new_creds)?;
    
    todo!("Implement credential rotation")
}
```

---

## Best Practices

### 1. Instance Initialization

✅ **Do:**
- Initialize each instance once
- Save credentials immediately to secure storage
- Verify credentials work before discarding output
- Document where credentials are stored

❌ **Don't:**
- Initialize the same instance multiple times
- Lose credentials (they can't be recovered!)
- Store credentials in code or config files
- Log plaintext credentials

### 2. Root User Usage

✅ **Do:**
- Use root for initial setup only
- Create admin users with specific policies
- Delegate day-to-day operations to regular users
- Keep root credentials in secure vault

❌ **Don't:**
- Use root for routine operations
- Share root credentials across teams
- Hard-code root credentials
- Give root access to applications

### 3. Credential Storage

✅ **Do:**
- Use secrets managers (AWS/Vault/etc.)
- Encrypt at rest
- Control access with IAM/RBAC
- Enable audit logging
- Rotate credentials regularly

❌ **Don't:**
- Commit to version control
- Store in plaintext files
- Log plaintext secrets
- Share via email/chat
- Store in application config

### 4. Authentication

✅ **Do:**
- Always authenticate before operations
- Use context for all service calls
- Validate credentials on each request
- Implement rate limiting
- Log authentication attempts

❌ **Don't:**
- Cache credentials in memory long-term
- Bypass authentication for "convenience"
- Reuse contexts across different users
- Ignore authentication failures

---

## Common Patterns

### Pattern 1: Application Startup

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize store
    let store = Arc::new(RwLock::new(InMemoryStore::new()));
    
    // 2. Get instance ID from config
    let instance_id = std::env::var("WAMI_INSTANCE_ID")?;
    
    // 3. Check if instance is initialized
    if !InstanceBootstrap::is_initialized(store.clone(), &instance_id).await? {
        // Initialize (first run)
        let creds = InstanceBootstrap::initialize_instance(
            store.clone(),
            &instance_id,
        ).await?;
        
        println!("⚠️  NEW INSTANCE - SAVE THESE CREDENTIALS:");
        println!("Access Key: {}", creds.access_key_id);
        println!("Secret Key: {}", creds.secret_access_key);
        
        // Store in secrets manager
        store_to_secrets_manager(&creds)?;
    }
    
    // 4. Retrieve credentials
    let creds = retrieve_from_secrets_manager(&instance_id)?;
    
    // 5. Authenticate
    let auth = AuthenticationService::new(store.clone());
    let context = auth
        .authenticate(&creds.access_key_id, &creds.secret_access_key)
        .await?;
    
    // 6. Run application
    run_app(store, context).await?;
    
    Ok(())
}
```

### Pattern 2: Multi-Tenant Application

```rust
async fn handle_request(
    store: Arc<RwLock<InMemoryStore>>,
    access_key_id: String,
    secret_key: String,
) -> Result<Response, AppError> {
    // 1. Authenticate user (determines tenant automatically)
    let auth = AuthenticationService::new(store.clone());
    let context = auth
        .authenticate(&access_key_id, &secret_key)
        .await?;
    
    // 2. Context now contains tenant information
    println!("User from tenant: {}", context.tenant_path());
    
    // 3. Operations are automatically scoped to tenant
    let user_service = UserService::new(store);
    let users = user_service.list_users(&context).await?;
    
    // Users only see resources in their tenant!
    Ok(Response::new(users))
}
```

### Pattern 3: Testing with Root User

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    async fn setup_test_instance() -> (Arc<RwLock<InMemoryStore>>, WamiContext) {
        let store = Arc::new(RwLock::new(InMemoryStore::new()));
        
        // Bootstrap test instance
        let creds = InstanceBootstrap::initialize_instance(
            store.clone(),
            "test-instance-123",
        ).await.unwrap();
        
        // Authenticate as root
        let auth = AuthenticationService::new(store.clone());
        let context = auth
            .authenticate(&creds.access_key_id, &creds.secret_access_key)
            .await.unwrap();
        
        (store, context)
    }
    
    #[tokio::test]
    async fn test_user_creation() {
        let (store, context) = setup_test_instance().await;
        
        // Use authenticated context in tests
        let service = UserService::new(store);
        let user = service.create_user(&context, request).await.unwrap();
        
        assert_eq!(user.user_name, "test-user");
    }
}
```

---

## Troubleshooting

### Issue: "Authentication failed"

**Cause:** Invalid credentials or wrong instance

**Solution:**
1. Verify access key ID and secret key are correct
2. Check that credentials match the instance ID
3. Ensure credentials haven't been rotated
4. Verify secrets manager is accessible

```rust
// Debug authentication
let result = auth_service
    .authenticate(&access_key_id, &secret_key)
    .await;

match result {
    Ok(context) => println!("Success: {}", context.caller_arn()),
    Err(e) => println!("Error: {:?}", e),
}
```

### Issue: "Instance not initialized"

**Cause:** Trying to authenticate before bootstrap

**Solution:**
```rust
// Check initialization first
if !InstanceBootstrap::is_initialized(store.clone(), instance_id).await? {
    let creds = InstanceBootstrap::initialize_instance(
        store.clone(),
        instance_id,
    ).await?;
    
    save_credentials(&creds)?;
}
```

### Issue: "Lost root credentials"

**Cause:** Credentials shown once and not saved

**Solution:**
- **If instance has other admin users:** Use their credentials to create new root access key
- **If no other access:** Must re-initialize instance (data loss!)

```rust
// Recovery with admin user
let admin_context = auth.authenticate(&admin_key, &admin_secret).await?;

// Create new root access key
// (Requires AccessKeyService implementation)
```

### Issue: "Credentials work but operations fail"

**Cause:** Authorization failure (policies)

**Solution:**
- Check if user has required policies attached
- Verify permissions boundaries
- Root users bypass authorization - use for testing

```rust
// Check authorization
let authz = AuthorizationService::new(store.clone());
let allowed = authz.is_authorized(
    &context,
    "iam:CreateUser",
    &resource_arn,
).await?;

println!("Operation allowed: {}", allowed);
```

---

## See Also

- [Authentication Guide](./AUTHENTICATION_GUIDE.md) - Detailed authentication concepts
- [Authorization Guide](./AUTHORIZATION_GUIDE.md) - Policy-based access control
- [Context Guide](./CONTEXT_GUIDE.md) - Using WamiContext
- [Security Best Practices](./SECURITY.md) - Comprehensive security guide
- [Example 26](../examples/26_secure_instance_bootstrap.rs) - Complete working example

---

**Last Updated:** October 30, 2025  
**Version:** 2.0.0+

