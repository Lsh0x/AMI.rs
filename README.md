# WAMI - Who Am I

**Multi-cloud Identity and Access Management Library for Rust**

[![GitHub last commit](https://img.shields.io/github/last-commit/lsh0x/wami)](https://github.com/lsh0x/wami/commits/main)
[![CI](https://github.com/lsh0x/wami/workflows/CI/badge.svg)](https://github.com/lsh0x/wami/actions)
[![Codecov](https://codecov.io/gh/lsh0x/wami/branch/main/graph/badge.svg)](https://codecov.io/gh/lsh0x/wami)
[![Docs](https://docs.rs/wami/badge.svg)](https://docs.rs/wami)
[![Crates.io](https://img.shields.io/crates/v/wami.svg)](https://crates.io/crates/wami)

## Overview

**WAMI** (Who Am I) is a pure Rust library for Identity and Access Management (IAM), Security Token Service (STS), and Single Sign-On (SSO) operations across multiple cloud providers. Built with a **domain-driven design**, WAMI separates business logic from storage, making it flexible, testable, and cloud-agnostic.

**Key Features:**
- 🌐 **Multi-cloud Support** - AWS, GCP, Azure, and custom identity providers
- 🏗️ **Pure Domain Logic** - Business logic without storage dependencies
- 💾 **Pluggable Storage** - In-memory, SQL, NoSQL, or custom backends
- 🏢 **Multi-tenant Architecture** - Built-in hierarchical tenant isolation
- 🔐 **Complete IAM Suite** - Users, groups, roles, policies, credentials
- 🔑 **Temporary Credentials** - STS sessions and role assumption
- 📊 **SSO Administration** - Permission sets, assignments, and federation
- 🦀 **100% Rust** - Type-safe, async-first, zero-cost abstractions
- ✅ **Well-tested** - 256+ unit tests with high coverage

---

## 📚 Documentation

### Getting Started
- **[Getting Started Guide](docs/GETTING_STARTED.md)** - Step-by-step tutorial for your first WAMI app
- **[Examples](docs/EXAMPLES.md)** - Complete working examples and common patterns

### Core Concepts
- **[Architecture](docs/ARCHITECTURE.md)** - Design principles, components, and data flow
- **[API Reference](docs/API_REFERENCE.md)** - Detailed API documentation for all modules

### Feature Guides
- **[IAM Guide](docs/IAM_GUIDE.md)** - Users, groups, roles, and policies
- **[STS Guide](docs/STS_GUIDE.md)** - Temporary credentials and sessions
- **[SSO Admin Guide](docs/SSO_ADMIN_GUIDE.md)** - Permission sets and account assignments
- **[Multi-tenant Guide](docs/MULTI_TENANT_GUIDE.md)** - Tenant isolation and hierarchy

### Advanced Topics
- **[Store Implementation](docs/STORE_IMPLEMENTATION.md)** - Create custom storage backends
- **[Multi-cloud Providers](docs/MULTICLOUD_PROVIDERS.md)** - AWS, GCP, Azure provider details
- **[Permission Checking](docs/PERMISSION_CHECKING.md)** - Policy evaluation and authorization
- **[ARN Architecture](docs/ARN_ARCHITECTURE_COMPLETE.md)** - Resource naming across providers

### Project Information
- **[Changelog](docs/CHANGELOG.md)** - Version history and release notes
- **[Multi-cloud Status](docs/MULTICLOUD_STATUS.md)** - Provider implementation status
- **[Security](docs/SECURITY.md)** - Security policies and vulnerability reporting

---

## Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wami = "0.8.0"
tokio = { version = "1.0", features = ["full"] }
```

### Your First Example

```rust
use wami::wami::identity::user::builder;
use wami::store::memory::InMemoryWamiStore;
use wami::store::traits::UserStore;
use wami::provider::aws::AwsProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize storage
    let mut store = InMemoryWamiStore::new();
    let provider = AwsProvider::new();
    
    // Build a user (pure function)
    let user = builder::build_user(
        "alice".to_string(),
        Some("/engineering/".to_string()),
        &provider,
        "123456789012"
    );
    
    // Store it
    let created = store.create_user(user).await?;
    println!("✅ Created user: {}", created.arn);
    
    // Retrieve it
    let retrieved = store.get_user("alice").await?;
    println!("✅ Retrieved: {:?}", retrieved.unwrap().user_name);
    
    Ok(())
}
```

**Output:**
```
✅ Created user: arn:aws:iam::123456789012:user/engineering/alice
✅ Retrieved: "alice"
```

See **[Getting Started Guide](docs/GETTING_STARTED.md)** for more examples.

---

## Architecture Overview

WAMI follows a clean 3-layer architecture:

```
┌─────────────────────────────────────────────────┐
│         Application Layer (Your Code)           │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────┼────────────────────────────┐
│    Domain Layer    │  Pure Functions            │
│                    │  (wami::*)                 │
│  • Identity        │  No storage dependencies   │
│  • Credentials     │  Pure business logic       │
│  • Policies        │  Builders & validators     │
│  • STS Sessions    │                            │
│  • Tenants         │                            │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────┼────────────────────────────┐
│    Storage Layer   │  Traits & Implementations  │
│  WamiStore         │  In-memory, SQL, custom    │
│  StsStore          │  Pluggable backends        │
│  TenantStore       │                            │
└─────────────────────────────────────────────────┘
```

**Key Benefits:**
- ✅ **Separation of Concerns** - Domain logic independent from storage
- ✅ **Testability** - Pure functions are easy to test
- ✅ **Flexibility** - Use any storage backend (memory, SQL, NoSQL)
- ✅ **Type Safety** - Rust's type system prevents common errors

Read more in **[Architecture Guide](docs/ARCHITECTURE.md)**.

---

## Core Features

### 🔐 Identity Management

```rust
use wami::wami::identity::{user, group, role};

// Create user
let user = user::builder::build_user("alice".into(), None, &provider, account);
store.create_user(user).await?;

// Create group and add user
let group = group::builder::build_group("admins".into(), None, &provider, account);
store.create_group(group).await?;
store.add_user_to_group("admins", "alice").await?;

// Create role with trust policy
let role = role::builder::build_role(
    "AdminRole".into(),
    trust_policy,
    None, None, None,
    &provider, account
);
store.create_role(role).await?;
```

### 🔑 Credentials & STS

```rust
use wami::wami::credentials::access_key;
use wami::wami::sts::session;

// Create access keys
let key = access_key::builder::build_access_key("alice".into(), &provider, account);
store.create_access_key(key).await?;

// Create temporary session
let session = session::builder::build_session(
    "session-123".into(),
    "AKIA...".into(),
    "secret".into(),
    3600, // 1 hour
    Some(role_arn),
    &provider, account
);
store.create_session(session).await?;
```

### 🏢 Multi-tenant Support

```rust
use wami::wami::tenant::{Tenant, TenantId};

// Create parent tenant
let parent_id = TenantId::root("acme-corp");
let parent = Tenant { id: parent_id.clone(), /* ... */ };
store.create_tenant(parent).await?;

// Create child tenant
let child_id = parent_id.child("engineering");
let child = Tenant { id: child_id.clone(), parent_id: Some(parent_id), /* ... */ };
store.create_tenant(child).await?;

// Query hierarchy
let descendants = store.get_descendants(&parent_id).await?;
```

---

## Project Structure

```
wami/
├── src/
│   ├── wami/              # Domain layer (pure functions)
│   │   ├── identity/      # Users, groups, roles
│   │   ├── credentials/   # Access keys, MFA, certificates
│   │   ├── policies/      # IAM policies
│   │   ├── sts/          # Sessions, temporary credentials
│   │   ├── sso_admin/    # SSO configuration
│   │   └── tenant/       # Multi-tenant models
│   │
│   ├── store/            # Storage layer
│   │   ├── traits/       # Storage trait definitions
│   │   └── memory/       # In-memory implementations
│   │
│   ├── provider/         # Cloud provider abstractions
│   │   ├── aws.rs
│   │   ├── gcp.rs
│   │   └── azure.rs
│   │
│   └── error.rs          # Error types
│
├── docs/                 # 📚 All documentation
├── examples/             # Working code examples
└── tests/                # Integration tests
```

---

## Testing

Run the full test suite:

```bash
cargo test
```

WAMI has **256+ tests** covering:
- ✅ Domain logic (pure functions)
- ✅ Store implementations (CRUD, queries, concurrency)
- ✅ Multi-tenant isolation
- ✅ Resource enumeration and downcasting

---

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Add tests for your changes
4. Ensure all tests pass (`cargo test`)
5. Run `cargo clippy` and `cargo fmt`
6. Submit a pull request

See **[Contributing Guide](CONTRIBUTING.md)** for more details.

---

## Roadmap

- [ ] SQL store implementations (PostgreSQL, MySQL)
- [ ] Policy evaluation engine
- [ ] SSO SAML integration
- [ ] Federation and external identity providers
- [ ] Audit logging and compliance
- [ ] Service/orchestration layer

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Security

Found a security issue? Please see **[Security Policy](docs/SECURITY.md)** for reporting guidelines.

---

## Links

- **[Documentation](docs/)** - Complete documentation index
- **[Crates.io](https://crates.io/crates/wami)** - Published crate
- **[Docs.rs](https://docs.rs/wami)** - API documentation
- **[GitHub](https://github.com/lsh0x/wami)** - Source code
- **[Issues](https://github.com/lsh0x/wami/issues)** - Bug reports and feature requests

---

<div align="center">
  <strong>Built with ❤️ in Rust</strong>
  <br>
  <sub>Multi-cloud IAM made simple</sub>
</div>
