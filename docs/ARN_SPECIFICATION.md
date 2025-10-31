# WAMI ARN Specification

## Overview

WAMI (Web Access Management Interface) uses a comprehensive ARN (Amazon Resource Name) format that supports multi-tenant hierarchies and multi-cloud provider mapping. This specification defines the ARN format, its components, and usage patterns.

## ARN Format

### WAMI Native ARN (No Cloud Sync)

For resources that exist only within WAMI and are not synced to any cloud provider:

```
arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{resource_type}/{resource_id}
```

**Example:**
```
arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755
```

### Cloud-Synced ARN

For resources that are synced with a cloud provider (AWS, GCP, Azure, Scaleway, etc.):

```
arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{provider}:{provider_account_id}:{resource_type}/{resource_id}
```

**Examples:**
```
arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:user/77557755
arn:wami:iam:t1/t2/t3:wami:999888777:gcp:554433221:user/77557755
arn:wami:iam:t1/t2/t3:wami:999888777:azure:sub-12345:user/77557755
arn:wami:iam:t1/t2/t3:wami:999888777:scaleway:112233445:user/77557755
```

## Component Breakdown

### 1. Prefix: `arn:wami`

All WAMI ARNs start with `arn:wami` to identify them as WAMI resource names.

### 2. Service

The WAMI service that manages the resource. Common values:

- `iam` - Identity and Access Management
- `sts` - Security Token Service
- `sso-admin` - SSO Administration
- Custom service names are also supported

**Example:**
```
arn:wami:iam:...
arn:wami:sts:...
arn:wami:sso-admin:...
```

### 3. Tenant Path

The hierarchical tenant path, with segments separated by `/`. This supports multi-tenant architectures with nested tenant hierarchies.

**Formats:**
- Single tenant: `t1`
- Two-level hierarchy: `t1/t2`
- Three-level hierarchy: `t1/t2/t3`
- Arbitrary depth: `root/dept/team/project`

**Examples:**
```
arn:wami:iam:t1:wami:...                  # Single tenant
arn:wami:iam:t1/t2:wami:...               # Two-level hierarchy
arn:wami:iam:t1/t2/t3:wami:...            # Three-level hierarchy
arn:wami:iam:acme/engineering/backend:... # Named hierarchy
```

**Benefits:**
- Easy resource scoping by tenant or sub-tenant
- Query all resources under a tenant prefix
- Support for organizational hierarchies
- Isolation between tenant branches

### 4. WAMI Marker: `wami`

A fixed marker to separate the tenant path from the instance ID. This ensures consistent parsing and avoids ambiguity.

### 5. WAMI Instance ID

A unique identifier for the WAMI deployment/instance. This allows multiple WAMI instances to coexist and distinguish their resources.

**Examples:**
```
arn:wami:iam:t1:wami:999888777:...
arn:wami:iam:t1:wami:prod-001:...
arn:wami:iam:t1:wami:dev-123:...
```

**Benefits:**
- Multi-instance deployments
- Environment separation (dev, staging, prod)
- Resource migration between instances
- Global resource identification

### 6. Cloud Provider Mapping with Region (Optional)

For cloud-synced resources, this section identifies the target cloud provider, account ID, and region.

**Format:** `{provider}:{provider_account_id}:{region}`

**Supported Providers:**
- `aws` - Amazon Web Services (regions: us-east-1, eu-west-1, etc.)
- `gcp` - Google Cloud Platform (regions: us-central1, europe-west1, etc.)
- `azure` - Microsoft Azure (regions: eastus, westeurope, etc.)
- `scaleway` - Scaleway (regions: fr-par, nl-ams, etc.)
- Custom provider names

**Examples:**
```
aws:223344556677:us-east-1
aws:223344556677:global        # For global services
gcp:554433221:us-central1
azure:sub-12345-67890:eastus
scaleway:112233445:fr-par
```

**Benefits:**
- **Multi-regional support:** Track resources across different regions
- **Region-specific queries:** Filter resources by region
- **Tenant-first design:** Query all regions in a tenant easily
- **Global service support:** Use "global" for region-independent resources
- **Cost optimization:** Track resources by region for billing

### 7. Resource

The resource type and ID, separated by `/`.

**Format:** `{resource_type}/{resource_id}`

**Resource Types (IAM):**
- `user` - IAM user
- `role` - IAM role
- `group` - IAM group
- `policy` - IAM policy
- `access-key` - Access key
- `mfa-device` - MFA device

**Resource Types (STS):**
- `session` - Session token
- `assumed-role` - Assumed role session
- `federated-user` - Federated user

**Resource Types (SSO Admin):**
- `instance` - SSO instance
- `permission-set` - Permission set
- `account-assignment` - Account assignment

**Resource ID:**
- Must be a stable identifier (not the resource name)
- Typically a numeric or alphanumeric ID
- Can contain `/` for hierarchical resources (e.g., policies)

**Examples:**
```
user/77557755
role/12345678
policy/98765432
policy/path/to/policy/123456
session/sess-abc123
```

## ARN Prefix Pattern

The ARN prefix is everything before the resource part. This is useful for querying and filtering resources.

**WAMI Native Prefix:**
```
arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}
```

**Cloud-Synced Prefix:**
```
arn:wami:{service}:{tenant_path}:wami:{wami_instance_id}:{provider}:{provider_account_id}
```

### Query Patterns

You can query resources by matching ARN prefixes:

1. **All resources in a WAMI instance:**
   ```
   arn:wami:iam:t1/t2/t3:wami:999888777
   ```
   Matches all IAM resources in instance `999888777` under tenant `t1/t2/t3`.

2. **All resources in a tenant (any subtenant):**
   ```
   arn:wami:iam:t1/t2
   ```
   Matches resources in `t1/t2`, `t1/t2/t3`, `t1/t2/t3/t4`, etc.

3. **All cloud-synced resources for a provider:**
   ```
   arn:wami:iam:t1:wami:999888777:aws:223344556677
   ```
   Matches all AWS-synced resources in the specified account.

4. **All resources of a specific type:**
   ```
   arn:wami:iam:t1/t2/t3:wami:999888777:user/
   ```
   Matches all users (both native and cloud-synced).

## Multi-Tenant Hierarchy

WAMI's ARN format natively supports hierarchical multi-tenancy.

### Example Hierarchy

```
Organization (t1)
├── Engineering (t1/eng)
│   ├── Backend (t1/eng/backend)
│   └── Frontend (t1/eng/frontend)
└── Marketing (t1/marketing)
    └── Content (t1/marketing/content)
```

### ARN Examples

```
arn:wami:iam:t1:wami:999888777:user/1001                    # Org-level user
arn:wami:iam:t1/eng:wami:999888777:user/2001                # Engineering user
arn:wami:iam:t1/eng/backend:wami:999888777:user/3001        # Backend team user
arn:wami:iam:t1/marketing/content:wami:999888777:user/4001  # Content team user
```

### Tenant Queries

- Query all Engineering resources: `arn:wami:iam:t1/eng`
  - Includes: `t1/eng`, `t1/eng/backend`, `t1/eng/frontend`
- Query all Backend resources: `arn:wami:iam:t1/eng/backend`
  - Includes only: `t1/eng/backend`

## Cloud Provider Transformation

WAMI ARNs can be transformed to provider-specific formats when syncing resources.

### AWS Transformation

**WAMI ARN:**
```
arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:user/77557755
```

**AWS ARN:**
```
arn:aws:iam::223344556677:user/77557755
```

**Mapping:**
- Drop WAMI-specific context (tenant path, instance ID)
- Use AWS format: `arn:aws:{service}::{account_id}:{resource}`
- Service mapping: `iam` → `iam`, `sts` → `sts`, `sso-admin` → `sso`

### GCP Transformation

**WAMI ARN:**
```
arn:wami:iam:t1/t2/t3:wami:999888777:gcp:554433221:serviceAccount/77557755
```

**GCP Resource Name:**
```
//iam.googleapis.com/projects/554433221/serviceAccounts/77557755
```

**Mapping:**
- GCP uses "resource names" not ARNs
- Format: `//{service}.googleapis.com/projects/{project_id}/{resource_type}s/{resource_id}`
- Service mapping: `iam` → `iam.googleapis.com`, `sso-admin` → `cloudidentity.googleapis.com`

### Azure Transformation

**WAMI ARN:**
```
arn:wami:iam:t1/t2/t3:wami:999888777:azure:sub-12345:user/77557755
```

**Azure Resource ID:**
```
/subscriptions/sub-12345/resourceGroups/wami-resources/providers/Microsoft.Authorization/user/77557755
```

**Mapping:**
- Azure uses hierarchical resource IDs
- Format: `/subscriptions/{subscription_id}/resourceGroups/{rg}/providers/{namespace}/{type}/{id}`
- Service mapping: `iam` → `Microsoft.Authorization`, `sso-admin` → `Microsoft.AzureActiveDirectory`

### Scaleway Transformation

**WAMI ARN:**
```
arn:wami:iam:t1/t2/t3:wami:999888777:scaleway:112233445:user/77557755
```

**Scaleway Resource:**
```
scw:112233445:iam:user/77557755
```

**Mapping:**
- Scaleway uses a simplified format
- Format: `scw:{organization_id}:{service}:{resource_type}/{resource_id}`

## Usage Examples

### Building ARNs

```rust
use wami::arn::{WamiArn, Service};

// WAMI native ARN
let arn = WamiArn::builder()
    .service(Service::Iam)
    .tenant_hierarchy(vec!["t1", "t2", "t3"])
    .wami_instance("999888777")
    .resource("user", "77557755")
    .build()?;

println!("{}", arn);
// Output: arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755

// Cloud-synced ARN
let arn = WamiArn::builder()
    .service(Service::Iam)
    .tenant("t1")
    .wami_instance("999888777")
    .cloud_provider("aws", "223344556677")
    .resource("user", "77557755")
    .build()?;

println!("{}", arn);
// Output: arn:wami:iam:t1:wami:999888777:aws:223344556677:user/77557755
```

### Parsing ARNs

```rust
use wami::arn::WamiArn;
use std::str::FromStr;

let arn = WamiArn::from_str("arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755")?;

println!("Service: {}", arn.service);
println!("Tenant: {}", arn.full_tenant_path());
println!("Instance: {}", arn.wami_instance_id);
println!("Resource Type: {}", arn.resource_type());
println!("Resource ID: {}", arn.resource_id());
println!("Cloud Synced: {}", arn.is_cloud_synced());
```

### Transforming to Provider Formats

```rust
use wami::arn::{WamiArn, AwsArnTransformer, ArnTransformer};

let arn = WamiArn::from_str("arn:wami:iam:t1:wami:999888777:aws:223344556677:user/77557755")?;

let transformer = AwsArnTransformer;
let aws_arn = transformer.to_provider_arn(&arn)?;

println!("{}", aws_arn);
// Output: arn:aws:iam::223344556677:user/77557755
```

### Querying Resources by Prefix

```rust
use wami::arn::WamiArn;

let arn = WamiArn::from_str("arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755")?;

// Check if ARN belongs to a specific tenant
let tenant_prefix = "arn:wami:iam:t1/t2";
if arn.matches_prefix(tenant_prefix) {
    println!("ARN belongs to tenant t1/t2 or its descendants");
}

// Check if ARN belongs to a specific instance
let instance_prefix = "arn:wami:iam:t1/t2/t3:wami:999888777";
if arn.matches_prefix(instance_prefix) {
    println!("ARN belongs to instance 999888777");
}
```

### Tenant Hierarchy Operations

```rust
use wami::arn::{WamiArn, TenantPath};

let parent_path = TenantPath::new(vec!["t1".to_string(), "t2".to_string()]);
let child_path = TenantPath::new(vec!["t1".to_string(), "t2".to_string(), "t3".to_string()]);

// Check relationship
assert!(child_path.is_descendant_of(&parent_path));
assert!(parent_path.is_ancestor_of(&child_path));

// Use with ARN
let arn = WamiArn::from_str("arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755")?;
assert!(arn.belongs_to_tenant(&parent_path));
```

## Best Practices

### 1. Use Resource IDs, Not Names

Always use stable resource IDs in ARNs, not human-readable names:

✅ **Good:**
```
arn:wami:iam:t1:wami:999888777:user/77557755
```

❌ **Bad:**
```
arn:wami:iam:t1:wami:999888777:user/john.doe
```

**Reason:** Resource names can change, but IDs remain stable. This prevents ARN invalidation when resources are renamed.

### 2. Design Tenant Hierarchies Carefully

Plan your tenant hierarchy to match your organizational structure:

```
arn:wami:iam:company/division/department/team:wami:...
```

**Benefits:**
- Easy resource isolation
- Hierarchical access control
- Clear organizational boundaries
- Efficient querying by organizational unit

### 3. Use Consistent Instance IDs

Choose a consistent naming scheme for WAMI instance IDs:

- Environment-based: `prod-001`, `staging-001`, `dev-001`
- Region-based: `us-east-1-001`, `eu-west-1-001`
- Numeric: `999888777`, `123456789`

### 4. Query by Prefix for Efficient Filtering

Leverage ARN prefixes for efficient resource queries:

```rust
// Get all resources in a tenant
let tenant_prefix = "arn:wami:iam:t1/t2:wami:999888777";
let resources = store.query_by_prefix(tenant_prefix);

// Get all cloud-synced AWS resources
let aws_prefix = "arn:wami:iam:t1:wami:999888777:aws:";
let aws_resources = store.query_by_prefix(aws_prefix);
```

### 5. Maintain WAMI ARN as Source of Truth

Even for cloud-synced resources, always use the WAMI ARN as the primary identifier:

```rust
// Store WAMI ARN in database
let wami_arn = "arn:wami:iam:t1:wami:999888777:aws:223344556677:user/77557755";

// Transform to provider format when needed
let transformer = AwsArnTransformer;
let aws_arn = transformer.to_provider_arn(&WamiArn::from_str(wami_arn)?)?;

// Use AWS ARN for AWS API calls
aws_client.get_user(&aws_arn).await?;
```

## Migration Guide

### From Old ARN Format

If you're migrating from a simpler ARN format to this new multi-tenant, multi-cloud format:

**Old Format:**
```
arn:wami:iam::user/john.doe
```

**New Format:**
```
arn:wami:iam:default:wami:main:user/77557755
```

**Migration Steps:**

1. **Choose default tenant:** Use `default` or `root` for existing resources
2. **Choose instance ID:** Use `main` or `prod-001` for your primary instance
3. **Generate resource IDs:** Assign stable IDs to all resources
4. **Update references:** Replace all old ARNs with new format
5. **Add cloud mappings:** If syncing to cloud, add provider and account ID

### Backward Compatibility

To maintain backward compatibility during migration:

1. **Accept both formats:** Support parsing both old and new ARN formats
2. **Gradual migration:** Migrate resources in batches
3. **ARN aliases:** Maintain a mapping from old ARNs to new ARNs
4. **Deprecation period:** Give users time to update their code

## Validation Rules

When creating or parsing ARNs, the following rules must be enforced:

1. **Prefix:** Must start with `arn:wami:`
2. **Service:** Must not be empty
3. **Tenant Path:**
   - Must not be empty
   - Must not contain empty segments
   - Segments separated by `/`
4. **WAMI Marker:** Must be `wami`
5. **Instance ID:** Must not be empty
6. **Cloud Mapping:** (optional)
   - If present, both provider and account ID must not be empty
7. **Resource:**
   - Must follow format `{type}/{id}`
   - Both type and ID must not be empty

## Security Considerations

### 1. ARN Injection

Always validate and sanitize ARN components before using them in queries or APIs:

```rust
// Validate ARN before use
let arn = WamiArn::from_str(user_input)?;

// ARN is now validated and safe to use
store.get_resource(&arn)?;
```

### 2. Tenant Isolation

Ensure that users can only access resources within their authorized tenant hierarchy:

```rust
// Check if user is authorized for the tenant
if !arn.belongs_to_tenant(&user.authorized_tenant) {
    return Err(AmiError::Unauthorized);
}
```

### 3. Cross-Tenant Access

Be careful when implementing cross-tenant access:

```rust
// Explicitly check cross-tenant permissions
if arn.tenant_path != user.tenant_path {
    // Verify cross-tenant access is allowed
    check_cross_tenant_permission(&user, &arn)?;
}
```

## Glossary

- **ARN:** Amazon Resource Name, a standardized format for identifying resources
- **WAMI:** Web Access Management Interface
- **Tenant:** An isolated organizational unit within WAMI
- **Tenant Hierarchy:** Nested organizational structure (parent/child/grandchild)
- **Instance ID:** Unique identifier for a WAMI deployment
- **Cloud Mapping:** Association between a WAMI resource and a cloud provider resource
- **Resource ID:** Stable, unique identifier for a resource (not the name)
- **Provider:** Cloud service provider (AWS, GCP, Azure, Scaleway, etc.)

## References

- [AWS ARN Format](https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html)
- [GCP Resource Names](https://cloud.google.com/apis/design/resource_names)
- [Azure Resource IDs](https://docs.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules)

## Version History

- **v1.0** (2025-10-30): Initial ARN specification with multi-tenant and multi-cloud support

