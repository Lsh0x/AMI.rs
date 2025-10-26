# Multi-Cloud Providers Guide

Build cloud-agnostic IAM systems with WAMI's provider abstraction.

## Overview

WAMI supports multiple cloud providers out of the box:
- **AWS** - Amazon Web Services
- **GCP** - Google Cloud Platform
- **Azure** - Microsoft Azure
- **Custom** - Your own cloud or on-premise system

Each provider generates cloud-specific resource identifiers (ARNs, resource names, etc.) while maintaining a consistent API.

## Quick Start

### AWS Provider (Default)

```rust
use wami::{InMemoryStore, MemoryIamClient, CreateUserRequest};
use wami::provider::AwsProvider;
use std::sync::Arc;

let provider = Arc::new(AwsProvider::new());
let store = InMemoryStore::with_provider(provider);
let mut iam = MemoryIamClient::new(store);

let user = iam.create_user(CreateUserRequest {
    user_name: "alice".to_string(),
    path: None,
    permissions_boundary: None,
    tags: None,
}).await?;

println!("{}", user.data.unwrap().arn);
// → arn:aws:iam::123456789012:user/alice
```

### GCP Provider

```rust
use wami::provider::GcpProvider;

let provider = Arc::new(GcpProvider::new("my-project-id"));
let store = InMemoryStore::with_provider(provider);
let mut iam = MemoryIamClient::new(store);

let user = iam.create_user(CreateUserRequest {
    user_name: "alice".to_string(),
    path: None,
    permissions_boundary: None,
    tags: None,
}).await?;

println!("{}", user.data.unwrap().arn);
// → projects/my-project-id/serviceAccounts/alice@my-project-id.iam.gserviceaccount.com
```

### Azure Provider

```rust
use wami::provider::AzureProvider;

let provider = Arc::new(AzureProvider::new(
    "subscription-id",
    "resource-group"
));
let store = InMemoryStore::with_provider(provider);
let mut iam = MemoryIamClient::new(store);

let user = iam.create_user(CreateUserRequest {
    user_name: "alice".to_string(),
    path: None,
    permissions_boundary: None,
    tags: None,
}).await?;

println!("{}", user.data.unwrap().arn);
// → /subscriptions/subscription-id/resourceGroups/resource-group/providers/Microsoft.Authorization/users/alice
```

## Provider Interface

```rust
pub trait CloudProvider: Send + Sync {
    /// Provider name ("aws", "gcp", "azure", etc.)
    fn name(&self) -> &str;
    
    /// Generate resource identifier (ARN, resource name, etc.)
    fn generate_resource_identifier(
        &self,
        resource_type: &str,
        resource_name: &str,
        path: Option<&str>,
    ) -> String;
    
    /// Generate account identifier
    fn generate_account_id(&self) -> String;
    
    /// Maximum session duration (seconds)
    fn max_session_duration(&self) -> u32;
    
    /// Validate session duration
    fn validate_session_duration(&self, duration: u32) -> Result<()>;
    
    /// Maximum users per account
    fn max_users(&self) -> Option<u32>;
    
    /// Maximum roles per account
    fn max_roles(&self) -> Option<u32>;
    
    /// Maximum policies per account
    fn max_policies(&self) -> Option<u32>;
}
```

## Built-in Providers

### AWS Provider

```rust
use wami::provider::AwsProvider;

// Default configuration
let provider = AwsProvider::new();

// Custom account ID
let provider = AwsProvider::with_account_id("123456789012");
```

**Resource Format**: `arn:aws:{service}::{account}:{resource-type}/{resource-name}`

**Features**:
- AWS-compatible ARNs
- 12-digit account IDs
- Session duration: 900-43200 seconds
- Quota limits match AWS defaults

**Examples**:
- User: `arn:aws:iam::123456789012:user/alice`
- Role: `arn:aws:iam::123456789012:role/DataScientist`
- Policy: `arn:aws:iam::123456789012:policy/S3ReadOnly`

### GCP Provider

```rust
use wami::provider::GcpProvider;

// With project ID
let provider = GcpProvider::new("my-project");

// With custom domain
let provider = GcpProvider::with_domain("my-project", "example.com");
```

**Resource Format**: `projects/{project}/serviceAccounts/{name}@{project}.iam.gserviceaccount.com`

**Features**:
- GCP-style service account emails
- Project-based organization
- Session duration: 3600-43200 seconds
- GCP quota limits

**Examples**:
- User: `projects/my-project/serviceAccounts/alice@my-project.iam.gserviceaccount.com`
- Role: `projects/my-project/roles/DataScientist`

### Azure Provider

```rust
use wami::provider::AzureProvider;

let provider = AzureProvider::new("subscription-id", "resource-group");
```

**Resource Format**: `/subscriptions/{sub}/resourceGroups/{rg}/providers/Microsoft.Authorization/{type}/{name}`

**Features**:
- Azure Resource Manager paths
- Subscription-based organization
- Session duration: 3600-86400 seconds
- Azure quota limits

**Examples**:
- User: `/subscriptions/xxx/resourceGroups/rg/providers/Microsoft.Authorization/users/alice`
- Role: `/subscriptions/xxx/resourceGroups/rg/providers/Microsoft.Authorization/roleDefinitions/DataScientist`

## Custom Provider

### Basic Custom Provider

```rust
use wami::provider::{CloudProvider, ProviderConfig};
use wami::AmiError;
use async_trait::async_trait;

pub struct MyCloudProvider {
    account_id: String,
}

impl MyCloudProvider {
    pub fn new(account_id: String) -> Self {
        Self { account_id }
    }
}

#[async_trait]
impl CloudProvider for MyCloudProvider {
    fn name(&self) -> &str {
        "mycloud"
    }
    
    fn generate_resource_identifier(
        &self,
        resource_type: &str,
        resource_name: &str,
        path: Option<&str>,
    ) -> String {
        let path_prefix = path.unwrap_or("/");
        format!(
            "mycloud://{}:{}{}{}",
            self.account_id,
            resource_type,
            path_prefix,
            resource_name
        )
    }
    
    fn generate_account_id(&self) -> String {
        self.account_id.clone()
    }
    
    fn max_session_duration(&self) -> u32 {
        7200 // 2 hours
    }
    
    fn validate_session_duration(&self, duration: u32) -> Result<(), AmiError> {
        if duration < 900 || duration > 7200 {
            return Err(AmiError::InvalidParameter {
                message: "Duration must be 900-7200 seconds".to_string(),
            });
        }
        Ok(())
    }
    
    fn max_users(&self) -> Option<u32> {
        Some(10000)
    }
    
    fn max_roles(&self) -> Option<u32> {
        Some(5000)
    }
    
    fn max_policies(&self) -> Option<u32> {
        Some(2000)
    }
}

// Usage
let provider = Arc::new(MyCloudProvider::new("acme-corp".to_string()));
let store = InMemoryStore::with_provider(provider);
```

**Output**:
- User: `mycloud://acme-corp:user/alice`
- Role: `mycloud://acme-corp:role//DataScientist`

### Advanced Custom Provider

```rust
pub struct EnterpriseCloudProvider {
    org_id: String,
    region: String,
    environment: String,
}

impl EnterpriseCloudProvider {
    pub fn new(org_id: String, region: String, environment: String) -> Self {
        Self { org_id, region, environment }
    }
}

impl CloudProvider for EnterpriseCloudProvider {
    fn name(&self) -> &str {
        "enterprise"
    }
    
    fn generate_resource_identifier(
        &self,
        resource_type: &str,
        resource_name: &str,
        path: Option<&str>,
    ) -> String {
        // Format: ent://{org}/{region}/{env}/{type}/{path}/{name}
        format!(
            "ent://{}/{}/{}/{}{}{}",
            self.org_id,
            self.region,
            self.environment,
            resource_type,
            path.unwrap_or("/"),
            resource_name
        )
    }
    
    fn generate_account_id(&self) -> String {
        format!("{}-{}-{}", self.org_id, self.region, self.environment)
    }
    
    // ... other methods
}
```

**Output**:
- User: `ent://acme/us-east/prod/user//alice`
- Role: `ent://acme/us-east/prod/role//DataScientist`

## Multi-Cloud Usage

### Running Multiple Providers Simultaneously

```rust
use wami::{InMemoryStore, MemoryIamClient, CreateUserRequest};
use wami::provider::{AwsProvider, GcpProvider, AzureProvider};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // AWS
    let aws_store = InMemoryStore::with_provider(Arc::new(AwsProvider::new()));
    let mut aws_iam = MemoryIamClient::new(aws_store);
    
    // GCP
    let gcp_store = InMemoryStore::with_provider(Arc::new(GcpProvider::new("proj")));
    let mut gcp_iam = MemoryIamClient::new(gcp_store);
    
    // Azure
    let azure_store = InMemoryStore::with_provider(
        Arc::new(AzureProvider::new("sub", "rg"))
    );
    let mut azure_iam = MemoryIamClient::new(azure_store);
    
    let request = CreateUserRequest {
        user_name: "alice".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    };
    
    // Create same user in all clouds
    let aws_user = aws_iam.create_user(request.clone()).await?;
    let gcp_user = gcp_iam.create_user(request.clone()).await?;
    let azure_user = azure_iam.create_user(request).await?;
    
    println!("AWS:   {}", aws_user.data.unwrap().arn);
    println!("GCP:   {}", gcp_user.data.unwrap().arn);
    println!("Azure: {}", azure_user.data.unwrap().arn);
    
    Ok(())
}
```

### Unified Multi-Cloud Client

```rust
use std::collections::HashMap;

pub struct MultiCloudManager {
    clients: HashMap<String, MemoryIamClient>,
}

impl MultiCloudManager {
    pub fn new() -> Self {
        let mut clients = HashMap::new();
        
        // AWS
        let aws_store = InMemoryStore::with_provider(Arc::new(AwsProvider::new()));
        clients.insert("aws".to_string(), MemoryIamClient::new(aws_store));
        
        // GCP
        let gcp_store = InMemoryStore::with_provider(Arc::new(GcpProvider::new("proj")));
        clients.insert("gcp".to_string(), MemoryIamClient::new(gcp_store));
        
        Self { clients }
    }
    
    pub async fn create_user_everywhere(&mut self, user_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let request = CreateUserRequest {
            user_name: user_name.to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        };
        
        for (cloud, client) in &mut self.clients {
            match client.create_user(request.clone()).await {
                Ok(user) => println!("✓ {} created in {}", user_name, cloud),
                Err(e) => eprintln!("✗ {} failed in {}: {:?}", user_name, cloud, e),
            }
        }
        
        Ok(())
    }
}

// Usage
let mut manager = MultiCloudManager::new();
manager.create_user_everywhere("alice").await?;
```

## Provider Comparison

| Feature | AWS | GCP | Azure | Custom |
|---------|-----|-----|-------|--------|
| ARN Format | arn:aws:... | projects/... | /subscriptions/... | Your choice |
| Account ID | 12 digits | Project ID | Subscription | Your choice |
| Max Session | 43200s | 43200s | 86400s | Configurable |
| Quotas | AWS defaults | GCP defaults | Azure defaults | Configurable |
| Multi-tenant | Via paths | Via projects | Via resource groups | Your design |

## Best Practices

### 1. Use Provider Abstraction

```rust
// Good: Provider-agnostic code
fn create_standard_user<S: Store>(
    iam: &mut IamClient<S>,
    name: &str,
) -> Result<User> {
    iam.create_user(CreateUserRequest {
        user_name: name.to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    }).await
}

// Works with any provider!
```

### 2. Store Provider Configuration

```rust
#[derive(Serialize, Deserialize)]
struct AppConfig {
    cloud_provider: String,
    provider_config: serde_json::Value,
}

fn create_provider(config: &AppConfig) -> Arc<dyn CloudProvider> {
    match config.cloud_provider.as_str() {
        "aws" => Arc::new(AwsProvider::new()),
        "gcp" => {
            let project_id = config.provider_config["project_id"].as_str().unwrap();
            Arc::new(GcpProvider::new(project_id))
        }
        "azure" => {
            let sub = config.provider_config["subscription_id"].as_str().unwrap();
            let rg = config.provider_config["resource_group"].as_str().unwrap();
            Arc::new(AzureProvider::new(sub, rg))
        }
        _ => panic!("Unknown provider"),
    }
}
```

### 3. Test with Multiple Providers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    async fn test_with_provider(provider: Arc<dyn CloudProvider>) {
        let store = InMemoryStore::with_provider(provider);
        let mut iam = MemoryIamClient::new(store);
        
        let user = iam.create_user(CreateUserRequest {
            user_name: "test".to_string(),
            path: None,
            permissions_boundary: None,
            tags: None,
        }).await.unwrap();
        
        assert_eq!(user.data.unwrap().user_name, "test");
    }
    
    #[tokio::test]
    async fn test_all_providers() {
        test_with_provider(Arc::new(AwsProvider::new())).await;
        test_with_provider(Arc::new(GcpProvider::new("test"))).await;
        test_with_provider(Arc::new(AzureProvider::new("sub", "rg"))).await;
    }
}
```

## Next Steps

- **[Getting Started](GETTING_STARTED.md)** - Basic usage
- **[Store Implementation](STORE_IMPLEMENTATION.md)** - Custom storage
- **[Architecture](ARCHITECTURE.md)** - System design
- **[Examples](EXAMPLES.md)** - Code samples

## Support

Questions? Open an issue on [GitHub](https://github.com/lsh0x/wami/issues).

