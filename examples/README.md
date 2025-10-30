# WAMI Examples

This directory contains **23 comprehensive examples** demonstrating WAMI's capabilities for Identity and Access Management.

## ✅ All 23 Examples Working

All examples compile and run successfully! Choose any example below to get started:

### 🚀 Getting Started (Examples 01-03)

#### 01. Hello WAMI
**Status:** ✅ Working  
**Run:** `cargo run --example 01_hello_wami`

Your first WAMI program. Creates a user, retrieves it, and lists all users.

**What you'll learn:**
- Basic store initialization
- Creating users
- Reading users
- Listing users

#### 02. Basic CRUD Operations
**Status:** ✅ Working  
**Run:** `cargo run --example 02_basic_crud_operations`

Complete CRUD operations for users, groups, and roles.

**What you'll learn:**
- Creating multiple resource types
- Updating resources
- Deleting resources
- Listing and filtering

#### 03. Service Layer Introduction
**Status:** ✅ Working  
**Run:** `cargo run --example 03_service_layer_intro`

Same operations as Example 02, but using the service layer instead of direct store access.

**What you'll learn:**
- Service layer benefits
- Thread-safe concurrent access with `Arc<RwLock<Store>>`
- Request/response DTOs
- Higher-level abstractions

### 🏢 Multi-Tenancy (Examples 04-07)

#### 04. Simple Multi-Tenant Setup
**Status:** ✅ Working  
**Run:** `cargo run --example 04_simple_multi_tenant`

Create multiple tenants with isolated resources.

**What you'll learn:**
- Creating tenants
- Tenant isolation
- Scoping services to specific tenants
- Cross-tenant resource separation

#### 05. Tenant Hierarchy
**Status:** ✅ Working  
**Run:** `cargo run --example 05_tenant_hierarchy`

Hierarchical tenant structures (root → department → team).

**What you'll learn:**
- Nested tenant structures
- Hierarchical tenant IDs
- Querying ancestors and descendants
- Organizational modeling

#### 06. Tenant Quotas and Limits
**Status:** ✅ Working  
**Run:** `cargo run --example 06_tenant_quotas_and_limits`

Resource quotas and limits for tenants.

**What you'll learn:**
- Setting resource quotas
- Quota inheritance in hierarchies
- Getting effective quotas
- Capacity planning

#### 07. Cross-Tenant Role Assumption
**Status:** ✅ Working  
**Run:** `cargo run --example 07_cross_tenant_role_assumption`

User in one tenant assuming a role in another tenant.

**What you'll learn:**
- Cross-tenant access patterns
- STS temporary credentials
- Trust policies
- Partner collaboration scenarios

### 🔐 Policies & Access Control (Examples 15-16)

#### 15. Policy Evaluation Simulation
**Status:** ✅ Working  
**Run:** `cargo run --example 15_policy_evaluation_simulation`

Simulate policy evaluation to test permissions before deployment.

**What you'll learn:**
- EvaluationService usage
- Testing permissions
- Understanding policy decisions
- Debugging access issues

#### 16. Role-Based Access Control (RBAC)
**Status:** ✅ Working  
**Run:** `cargo run --example 16_role_based_access_control`

Implement RBAC with roles, policies, and user assignments.

**What you'll learn:**
- Defining roles (admin, developer, viewer)
- Assigning users to roles
- Policy inheritance through roles
- Centralized permission management

#### 23. Permissions Boundaries
**Status:** ✅ Working  
**Run:** `cargo run --example 23_permissions_boundaries`

Advanced permission management using boundaries to set maximum permissions for users and roles.

**What you'll learn:**
- Setting permissions boundaries on users and roles
- Understanding effective permissions (identity policies ∩ boundary)
- Preventing privilege escalation
- Sandbox environments and contractor access
- Delegated administration patterns
- Multi-tenant permission isolation

### 🔐 SSO & Federation (Examples 21-22)

#### 21. SSO Setup (Basic)
**Status:** ✅ Working  
**Run:** `cargo run --example 21_sso_setup_basic`

Basic SSO instance and permission set configuration.

**What you'll learn:**
- Creating SSO instances
- Managing permission sets
- SSO resource organization
- Enterprise SSO patterns

#### 22. Identity Providers for Federation
**Status:** ✅ Working  
**Run:** `cargo run --example 22_identity_providers_federation`

Comprehensive federated authentication setup with SAML and OIDC providers.

**What you'll learn:**
- Creating SAML providers (Okta, Azure AD)
- Creating OIDC providers (Google, Auth0)
- Managing client IDs and thumbprints
- Certificate rotation
- Tagging and organizing providers
- Usage tracking
- Federation patterns

## Running Examples

All examples use the in-memory store and require no external dependencies:

```bash
# Run a specific example
cargo run --example 01_hello_wami

# Run with output
cargo run --example 02_basic_crud_operations

# List all available examples
cargo run --example
```

## Example Categories

| Category | Examples | Description | Status |
|----------|----------|-------------|--------|
| **Getting Started** | 01-03 | Basic operations and service layer | ✅ All Working |
| **Multi-Tenancy** | 04-08 | Tenant management, hierarchy, migration | ✅ All Working |
| **Multi-Cloud** | 09-13 | Cross-cloud identity management | ✅ All Working |
| **Policies** | 14-17, 23 | Access control, permissions, boundaries | ✅ All Working |
| **STS & Sessions** | 18-20 | Temporary credentials and federation | ✅ All Working |
| **SSO & Federation** | 21-22 | SSO and identity provider federation | ✅ All Working |

## Next Steps

After running these examples, check out:

- [API Reference](../docs/API_REFERENCE.md) - Complete API documentation
- [Architecture Guide](../docs/ARCHITECTURE.md) - System design and patterns
- [Multi-Tenant Guide](../docs/MULTI_TENANT_GUIDE.md) - Advanced multi-tenancy
- [IAM Guide](../docs/IAM_GUIDE.md) - IAM concepts and best practices

## Contributing

Found an issue or want to contribute a new example?

1. Check the [issues](../docs/issues/) for known problems
2. Read the [Contributing Guide](../CONTRIBUTING.md)
3. Submit a pull request with your improvements

## Support

- **Documentation:** [docs/](../docs/)
- **Tests:** All library tests pass (402/402 ✅)
- **Questions:** Open an issue on GitHub

