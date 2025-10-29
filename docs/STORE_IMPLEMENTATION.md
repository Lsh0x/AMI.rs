# Store Implementation Guide

Learn how to create custom storage backends for WAMI.

## Overview

WAMI's storage layer is designed to be **pluggable**. You can implement your own storage backend (SQL, NoSQL, cloud-native) by implementing the storage traits.

**Key Benefits:**
- ✅ Use any database or storage system
- ✅ Keep the same API as in-memory store
- ✅ Leverage WAMI's domain logic
- ✅ Easy to test with trait bounds

---

## Architecture

```
Your Application
       ↓
Domain Functions (wami::*)
       ↓
Storage Traits (store::traits::*)
       ↓
┌──────────────┬──────────────┬──────────────┐
│   Memory     │   Your SQL   │  Your NoSQL  │
│   (built-in) │   (custom)   │   (custom)   │
└──────────────┴──────────────┴──────────────┘
```

---

## Quick Start

### Step 1: Choose Your Traits

WAMI has modular traits. Implement only what you need:

```rust
// Option A: Implement specific sub-traits
use wami::store::traits::{UserStore, GroupStore, RoleStore};

// Option B: Implement the composite trait
use wami::store::traits::WamiStore;  // All IAM sub-traits
```

### Step 2: Create Your Store Struct

```rust
use sqlx::PgPool;

pub struct PostgresWamiStore {
    pool: PgPool,
}

impl PostgresWamiStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### Step 3: Implement the Traits

```rust
use wami::store::traits::UserStore;
use wami::wami::identity::User;
use wami::error::Result;
use async_trait::async_trait;

#[async_trait]
impl UserStore for PostgresWamiStore {
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
            user.created_at
        )
        .execute(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn get_user(&self, user_name: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users WHERE user_name = $1
            "#,
            user_name
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn update_user(&mut self, user: User) -> Result<User> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET path = $2
            WHERE user_name = $1
            "#,
            user.user_name,
            user.path
        )
        .execute(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn delete_user(&mut self, user_name: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM users WHERE user_name = $1
            "#,
            user_name
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn list_users(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<User>, bool, Option<String>)> {
        let limit = pagination
            .and_then(|p| p.max_items)
            .unwrap_or(100) as i64;
        
        let users = if let Some(prefix) = path_prefix {
            sqlx::query_as!(
                User,
                r#"
                SELECT * FROM users 
                WHERE path LIKE $1
                LIMIT $2
                "#,
                format!("{}%", prefix),
                limit
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                User,
                r#"
                SELECT * FROM users LIMIT $1
                "#,
                limit
            )
            .fetch_all(&self.pool)
            .await?
        };
        
        let has_more = users.len() as i64 == limit;
        let next_marker = if has_more {
            users.last().map(|u| u.user_name.clone())
        } else {
            None
        };
        
        Ok((users, has_more, next_marker))
    }
    
    async fn tag_user(&mut self, user_name: &str, tags: Vec<Tag>) -> Result<()> {
        for tag in tags {
            sqlx::query!(
                r#"
                INSERT INTO user_tags (user_name, key, value)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_name, key) 
                DO UPDATE SET value = EXCLUDED.value
                "#,
                user_name,
                tag.key,
                tag.value
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
    
    async fn list_user_tags(&self, user_name: &str) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as!(
            Tag,
            r#"
            SELECT key, value FROM user_tags 
            WHERE user_name = $1
            "#,
            user_name
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(tags)
    }
    
    async fn untag_user(&mut self, user_name: &str, tag_keys: Vec<String>) -> Result<()> {
        for key in tag_keys {
            sqlx::query!(
                r#"
                DELETE FROM user_tags 
                WHERE user_name = $1 AND key = $2
                "#,
                user_name,
                key
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}

// Implement other sub-traits (GroupStore, RoleStore, etc.)
// ...

// Once all sub-traits are implemented, WamiStore is automatically implemented!
```

### Step 4: Use Your Custom Store

```rust
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:pass@localhost/wami")
        .await?;
    
    // Create your custom store
    let mut store = PostgresWamiStore::new(pool);
    
    // Use it exactly like InMemoryWamiStore!
    let provider = AwsProvider::new();
    let user = user::builder::build_user(
        "alice".to_string(),
        None,
        &provider,
        "123456789012"
    );
    
    store.create_user(user).await?;
    
    Ok(())
}
```

---

## Database Schema Example

### PostgreSQL Schema

```sql
-- Users table
CREATE TABLE users (
    user_name VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL UNIQUE,
    arn TEXT NOT NULL,
    wami_arn TEXT NOT NULL,
    path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id TEXT
);

CREATE INDEX idx_users_path ON users(path);
CREATE INDEX idx_users_tenant ON users(tenant_id);

-- Groups table
CREATE TABLE groups (
    group_name VARCHAR(128) PRIMARY KEY,
    group_id VARCHAR(128) NOT NULL UNIQUE,
    arn TEXT NOT NULL,
    wami_arn TEXT NOT NULL,
    path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id TEXT
);

-- Group memberships
CREATE TABLE group_members (
    group_name VARCHAR(128) NOT NULL REFERENCES groups(group_name) ON DELETE CASCADE,
    user_name VARCHAR(64) NOT NULL REFERENCES users(user_name) ON DELETE CASCADE,
    PRIMARY KEY (group_name, user_name)
);

CREATE INDEX idx_group_members_user ON group_members(user_name);

-- Roles table
CREATE TABLE roles (
    role_name VARCHAR(64) PRIMARY KEY,
    role_id VARCHAR(128) NOT NULL UNIQUE,
    arn TEXT NOT NULL,
    wami_arn TEXT NOT NULL,
    assume_role_policy_document TEXT NOT NULL,
    path TEXT,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    max_session_duration INTEGER,
    tenant_id TEXT
);

-- Access keys
CREATE TABLE access_keys (
    access_key_id VARCHAR(128) PRIMARY KEY,
    user_name VARCHAR(64) NOT NULL REFERENCES users(user_name) ON DELETE CASCADE,
    secret_access_key TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Active',
    wami_arn TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id TEXT
);

CREATE INDEX idx_access_keys_user ON access_keys(user_name);

-- Tags (for users, roles, etc.)
CREATE TABLE user_tags (
    user_name VARCHAR(64) NOT NULL REFERENCES users(user_name) ON DELETE CASCADE,
    key VARCHAR(128) NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (user_name, key)
);

CREATE TABLE role_tags (
    role_name VARCHAR(64) NOT NULL REFERENCES roles(role_name) ON DELETE CASCADE,
    key VARCHAR(128) NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (role_name, key)
);

-- Sessions (STS)
CREATE TABLE sts_sessions (
    session_token VARCHAR(256) PRIMARY KEY,
    access_key_id VARCHAR(128) NOT NULL,
    secret_access_key TEXT NOT NULL,
    expiration TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Active',
    assumed_role_arn TEXT,
    federated_user_name TEXT,
    principal_arn TEXT,
    arn TEXT NOT NULL,
    wami_arn TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used TIMESTAMPTZ,
    tenant_id TEXT
);

CREATE INDEX idx_sessions_expiration ON sts_sessions(expiration);
CREATE INDEX idx_sessions_status ON sts_sessions(status);

-- Tenants
CREATE TABLE tenants (
    id TEXT PRIMARY KEY,
    name VARCHAR(256) NOT NULL,
    parent_id TEXT REFERENCES tenants(id),
    organization TEXT,
    tenant_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Active',
    arn TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    max_child_depth INTEGER NOT NULL DEFAULT 5,
    can_create_sub_tenants BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_tenants_parent ON tenants(parent_id);
CREATE INDEX idx_tenants_status ON tenants(status);

-- Tenant quotas
CREATE TABLE tenant_quotas (
    tenant_id TEXT PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,
    max_users INTEGER NOT NULL,
    max_roles INTEGER NOT NULL,
    max_policies INTEGER NOT NULL,
    max_groups INTEGER NOT NULL,
    max_access_keys INTEGER NOT NULL,
    max_sub_tenants INTEGER NOT NULL,
    api_rate_limit INTEGER NOT NULL,
    quota_mode VARCHAR(20) NOT NULL DEFAULT 'Inherited'
);
```

---

## Implementation Tips

### 1. Error Handling

Convert database errors to `AmiError`:

```rust
use wami::error::AmiError;

impl From<sqlx::Error> for AmiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AmiError::ResourceNotFound {
                resource: "Resource not found".into(),
            },
            _ => AmiError::InternalError(err.to_string()),
        }
    }
}
```

### 2. Transactions

Use transactions for multi-step operations:

```rust
async fn add_user_to_group(&mut self, group_name: &str, user_name: &str) -> Result<()> {
    let mut tx = self.pool.begin().await?;
    
    // Verify user exists
    sqlx::query!("SELECT 1 FROM users WHERE user_name = $1", user_name)
        .fetch_one(&mut *tx)
        .await?;
    
    // Verify group exists
    sqlx::query!("SELECT 1 FROM groups WHERE group_name = $1", group_name)
        .fetch_one(&mut *tx)
        .await?;
    
    // Add membership
    sqlx::query!(
        "INSERT INTO group_members (group_name, user_name) VALUES ($1, $2)",
        group_name,
        user_name
    )
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    Ok(())
}
```

### 3. Connection Pooling

Always use connection pooling for production:

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(600))
    .connect(&database_url)
    .await?;
```

### 4. Indexing Strategy

Create indexes for common queries:

```sql
-- Path prefix queries
CREATE INDEX idx_users_path ON users(path text_pattern_ops);

-- Pagination
CREATE INDEX idx_users_name ON users(user_name);

-- Tenant isolation
CREATE INDEX idx_users_tenant ON users(tenant_id);

-- Composite indexes
CREATE INDEX idx_group_members_lookup ON group_members(user_name, group_name);
```

### 5. Tenant Isolation

Always filter by tenant_id:

```rust
async fn list_users(
    &self,
    path_prefix: Option<&str>,
    pagination: Option<&PaginationParams>,
) -> Result<(Vec<User>, bool, Option<String>)> {
    let tenant_id = self.current_tenant_id();  // Get from context
    
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users 
        WHERE tenant_id = $1 
        AND ($2::TEXT IS NULL OR path LIKE $2)
        LIMIT $3
        "#,
        tenant_id,
        path_prefix.map(|p| format!("{}%", p)),
        limit
    )
    .fetch_all(&self.pool)
    .await?;
    
    Ok((users, false, None))
}
```

---

## Testing Your Store

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    async fn setup() -> PostgresWamiStore {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgres://localhost/wami_test")
            .await
            .unwrap();
        
        PostgresWamiStore::new(pool)
    }
    
    #[tokio::test]
    async fn test_user_crud() {
        let mut store = setup().await;
        
        // Create
        let user = user::builder::build_user(
            "test-user".into(),
            None,
            &AwsProvider::new(),
            "123"
        );
        store.create_user(user).await.unwrap();
        
        // Read
        let retrieved = store.get_user("test-user").await.unwrap();
        assert!(retrieved.is_some());
        
        // Update
        let mut user = retrieved.unwrap();
        user.path = Some("/updated/".into());
        store.update_user(user).await.unwrap();
        
        // Delete
        store.delete_user("test-user").await.unwrap();
        assert!(store.get_user("test-user").await.unwrap().is_none());
    }
}
```

---

## Performance Optimization

### 1. Batch Operations

```rust
async fn create_users_batch(&mut self, users: Vec<User>) -> Result<Vec<User>> {
    let mut tx = self.pool.begin().await?;
    
    for user in &users {
        sqlx::query!(
            "INSERT INTO users (...) VALUES (...)",
            // user fields
        )
        .execute(&mut *tx)
        .await?;
    }
    
    tx.commit().await?;
    Ok(users)
}
```

### 2. Prepared Statements

Use parameterized queries for better performance:

```rust
// sqlx automatically prepares statements
let users = sqlx::query_as!(User, "SELECT * FROM users WHERE path = $1", path)
    .fetch_all(&self.pool)
    .await?;
```

### 3. Caching Layer

Add Redis caching for frequently accessed resources:

```rust
pub struct CachedPostgresStore {
    postgres: PostgresWamiStore,
    redis: RedisClient,
}

impl UserStore for CachedPostgresStore {
    async fn get_user(&self, user_name: &str) -> Result<Option<User>> {
        // Try cache first
        if let Ok(Some(cached)) = self.redis.get(user_name).await {
            return Ok(Some(cached));
        }
        
        // Fallback to database
        let user = self.postgres.get_user(user_name).await?;
        
        // Cache for next time
        if let Some(ref u) = user {
            self.redis.set(user_name, u, Duration::from_secs(300)).await?;
        }
        
        Ok(user)
    }
}
```

---

## See Also

- **[Architecture](ARCHITECTURE.md)** - WAMI's design principles
- **[API Reference](API_REFERENCE.md)** - Complete trait documentation
- **[Getting Started](GETTING_STARTED.md)** - Basic usage
- **[Examples](EXAMPLES.md)** - Working code examples
