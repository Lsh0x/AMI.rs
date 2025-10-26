# Store Implementation Guide

Learn how to implement custom storage backends for WAMI.

## Overview

WAMI uses a pluggable store architecture. The built-in `InMemoryStore` is great for testing, but for production you'll want to implement custom persistence (PostgreSQL, MongoDB, Redis, etc.).

## Architecture

```
Store (main trait)
├── IamStore       → Users, roles, policies, groups
├── StsStore       → Sessions, credentials
├── SsoAdminStore  → Permission sets, assignments
└── TenantStore    → Tenants, hierarchy
```

## Quick Start

### 1. Using In-Memory Store

```rust
use wami::{InMemoryStore, MemoryIamClient};

let store = InMemoryStore::new();
let mut iam = MemoryIamClient::new(store);

// Data stored in memory (lost on restart)
```

### 2. Custom Store Interface

```rust
use wami::store::{Store, IamStore, StsStore, SsoAdminStore};
use wami::tenant::store::TenantStore;
use async_trait::async_trait;

#[async_trait]
pub trait Store: Send + Sync {
    type IamStore: IamStore;
    type StsStore: StsStore;
    type SsoAdminStore: SsoAdminStore;
    type TenantStore: TenantStore;
    
    fn cloud_provider(&self) -> &dyn CloudProvider;
    
    async fn iam_store(&mut self) -> Result<&mut Self::IamStore>;
    async fn sts_store(&mut self) -> Result<&mut Self::StsStore>;
    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore>;
    async fn tenant_store(&mut self) -> Result<&mut Self::TenantStore>;
}
```

## Implementing a Custom Store

### Example: PostgreSQL Store

```rust
use sqlx::PgPool;
use wami::store::Store;
use std::sync::Arc;

pub struct PostgresStore {
    pool: PgPool,
    provider: Arc<dyn CloudProvider>,
    iam: PostgresIamStore,
    sts: PostgresStsStore,
    sso: PostgresSsoAdminStore,
    tenant: PostgresTenantStore,
}

impl PostgresStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        let provider = Arc::new(AwsProvider::new());
        
        Ok(Self {
            pool: pool.clone(),
            provider,
            iam: PostgresIamStore::new(pool.clone()),
            sts: PostgresStsStore::new(pool.clone()),
            sso: PostgresSsoAdminStore::new(pool.clone()),
            tenant: PostgresTenantStore::new(pool),
        })
    }
}

#[async_trait]
impl Store for PostgresStore {
    type IamStore = PostgresIamStore;
    type StsStore = PostgresStsStore;
    type SsoAdminStore = PostgresSsoAdminStore;
    type TenantStore = PostgresTenantStore;
    
    fn cloud_provider(&self) -> &dyn CloudProvider {
        self.provider.as_ref()
    }
    
    async fn iam_store(&mut self) -> Result<&mut Self::IamStore> {
        Ok(&mut self.iam)
    }
    
    async fn sts_store(&mut self) -> Result<&mut Self::StsStore> {
        Ok(&mut self.sts)
    }
    
    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore> {
        Ok(&mut self.sso)
    }
    
    async fn tenant_store(&mut self) -> Result<&mut Self::TenantStore> {
        Ok(&mut self.tenant)
    }
}
```

### IAM Store Implementation

```rust
use wami::iam::User;
use wami::store::traits::IamStore;

pub struct PostgresIamStore {
    pool: PgPool,
}

#[async_trait]
impl IamStore for PostgresIamStore {
    async fn create_user(&mut self, user: User) -> Result<User> {
        sqlx::query!(
            r#"
            INSERT INTO users (user_name, user_id, arn, path, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            user.user_name,
            user.user_id,
            user.arn,
            user.path,
            user.create_date
        )
        .execute(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn get_user(&self, user_name: &str) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            User,
            r#"SELECT * FROM users WHERE user_name = $1"#,
            user_name
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }
    
    // Implement other IamStore methods...
}
```

## Database Schema

### Users Table

```sql
CREATE TABLE users (
    user_name VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL UNIQUE,
    arn TEXT NOT NULL,
    path TEXT NOT NULL DEFAULT '/',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    password_last_used TIMESTAMPTZ,
    permissions_boundary TEXT,
    tags JSONB,
    wami_arn TEXT NOT NULL,
    providers JSONB,
    tenant_id TEXT
);

CREATE INDEX idx_users_path ON users(path);
CREATE INDEX idx_users_tenant ON users(tenant_id);
```

### Complete Schema

See `examples/postgres_store/migrations/` for complete database schema including:
- Users, roles, policies, groups
- Access keys, MFA devices, login profiles
- STS sessions and credentials
- SSO permission sets and assignments
- Tenants and hierarchy

## Using Your Custom Store

```rust
use wami::IamClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup your custom store
    let store = PostgresStore::new("postgresql://localhost/wami").await?;
    
    // Use with clients
    let mut iam = IamClient::new(store);
    
    // All operations now persist to PostgreSQL!
    let user = iam.create_user(CreateUserRequest {
        user_name: "alice".to_string(),
        path: None,
        permissions_boundary: None,
        tags: None,
    }).await?;
    
    Ok(())
}
```

## Best Practices

### 1. Transaction Support

```rust
async fn create_user_transactional(&mut self, user: User) -> Result<User> {
    let mut tx = self.pool.begin().await?;
    
    // Insert user
    sqlx::query!(/* ... */).execute(&mut tx).await?;
    
    // Insert tags
    for tag in &user.tags {
        sqlx::query!(/* ... */).execute(&mut tx).await?;
    }
    
    tx.commit().await?;
    Ok(user)
}
```

### 2. Connection Pooling

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(10))
    .connect("postgresql://...").await?;
```

### 3. Proper Indexing

```sql
-- Performance indexes
CREATE INDEX idx_users_path ON users(path);
CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_access_keys_user ON access_keys(user_name);
CREATE INDEX idx_sessions_expiration ON sts_sessions(expiration);
```

### 4. Error Mapping

```rust
impl From<sqlx::Error> for wami::AmiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => wami::AmiError::ResourceNotFound {
                resource: "Resource not found".to_string(),
            },
            _ => wami::AmiError::InternalError {
                message: format!("Database error: {}", err),
            },
        }
    }
}
```

### 5. Caching Layer

```rust
use moka::future::Cache;

pub struct CachedStore {
    store: PostgresStore,
    user_cache: Cache<String, User>,
}

impl CachedStore {
    async fn get_user(&self, user_name: &str) -> Result<Option<User>> {
        // Check cache first
        if let Some(user) = self.user_cache.get(user_name).await {
            return Ok(Some(user));
        }
        
        // Fetch from database and cache
        if let Some(user) = self.store.iam_store().await?.get_user(user_name).await? {
            self.user_cache.insert(user_name.to_string(), user.clone()).await;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
}
```

## Different Storage Strategies

You can mix storage backends for different domains:

```rust
pub struct HybridStore {
    iam_store: PostgresIamStore,      // PostgreSQL for IAM data
    sts_store: RedisStsStore,          // Redis for sessions (fast, expiring)
    sso_admin_store: PostgresSsoStore, // PostgreSQL for SSO
    tenant_store: PostgresTenantStore, // PostgreSQL for tenants
}
```

**Why mix backends?**
- **IAM**: Needs ACID compliance → PostgreSQL
- **STS Sessions**: Need TTL/expiration → Redis
- **SSO Admin**: Needs relations → PostgreSQL
- **Tenants**: Needs hierarchy queries → PostgreSQL

## Testing Your Store

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_and_get_user() {
        let store = PostgresStore::new("postgresql://localhost/wami_test")
            .await
            .unwrap();
        let mut iam = store.iam_store().await.unwrap();
        
        let user = User {
            user_name: "test-user".to_string(),
            user_id: "AIDATEST123".to_string(),
            arn: "arn:aws:iam::123456789012:user/test-user".to_string(),
            path: "/".to_string(),
            create_date: chrono::Utc::now(),
            password_last_used: None,
            permissions_boundary: None,
            tags: vec![],
            wami_arn: "arn:wami:iam::123456789012:user/test-user".to_string(),
            providers: vec![],
            tenant_id: None,
        };
        
        // Create
        let created = iam.create_user(user.clone()).await.unwrap();
        assert_eq!(created.user_name, "test-user");
        
        // Get
        let fetched = iam.get_user("test-user").await.unwrap().unwrap();
        assert_eq!(fetched.user_name, "test-user");
    }
}
```

## Resources

- **Store Traits**: `src/store/traits/`
- **In-Memory Reference**: `src/store/memory/`
- **Examples**: `examples/postgres_store/`
- **API Docs**: Run `cargo doc --open`

## Next Steps

- See [IamStore trait](../src/store/traits/iam.rs) for all required methods
- See [StsStore trait](../src/store/traits/sts.rs) for STS operations
- See [TenantStore trait](../src/tenant/store/mod.rs) for tenant operations
- Check `examples/` directory for complete implementations

## Support

Questions? Open an issue on [GitHub](https://github.com/lsh0x/wami/issues).

