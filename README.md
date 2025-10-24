# AMI.rs

AWS IAM, STS, and SSO Admin operations library for Rust

[![GitHub last commit](https://img.shields.io/github/last-commit/lsh0x/AMI.rs)](https://github.com/lsh0x/AMI.rs/commits/main)
[![CI](https://github.com/lsh0x/AMI.rs/workflows/CI/badge.svg)](https://github.com/lsh0x/AMI.rs/actions)
[![Codecov](https://codecov.io/gh/lsh0x/AMI.rs/branch/main/graph/badge.svg)](https://codecov.io/gh/lsh0x/AMI.rs)
[![Docs](https://docs.rs/ami/badge.svg)](https://docs.rs/ami)
[![Crates.io](https://img.shields.io/crates/v/ami.svg)](https://crates.io/crates/ami)
[![crates.io](https://img.shields.io/crates/d/ami)](https://crates.io/crates/ami)

## Overview

AMI.rs is a comprehensive Rust library that provides easy-to-use interfaces for AWS Identity and Access Management (IAM), Security Token Service (STS), and Single Sign-On Admin operations. This library abstracts the complexity of AWS SDK calls and provides a clean, type-safe API for managing AWS identities and permissions.

## Features

### 🔐 IAM Operations

<details>
<summary><strong>Users</strong></summary>

- `CreateUser` - Create a new IAM user
- `DeleteUser` - Delete an IAM user
- `GetUser` - Retrieve user information
- `UpdateUser` - Update user properties
- `ListUsers` - List all users
- `ListUserTags` - List tags for a user

</details>

<details>
<summary><strong>Access Keys</strong></summary>

- `CreateAccessKey` - Create access keys for a user
- `DeleteAccessKey` - Delete access keys
- `UpdateAccessKey` - Update access key status
- `ListAccessKeys` - List user's access keys
- `GetAccessKeyLastUsed` - Get last used information

</details>

<details>
<summary><strong>Passwords</strong></summary>

- `CreateLoginProfile` - Create console login profile
- `UpdateLoginProfile` - Update login profile
- `DeleteLoginProfile` - Delete login profile
- `GetLoginProfile` - Get login profile information

</details>

<details>
<summary><strong>MFA Devices</strong></summary>

- `EnableMFADevice` - Enable MFA device
- `DeactivateMFADevice` - Deactivate MFA device
- `ListMFADevices` - List MFA devices
- `ResyncMFADevice` - Resync MFA device

</details>

<details>
<summary><strong>Service Specific Credentials</strong></summary>

- `CreateServiceSpecificCredential` - Create service-specific credentials
- `UpdateServiceSpecificCredential` - Update service-specific credentials
- `DeleteServiceSpecificCredential` - Delete service-specific credentials
- `ListServiceSpecificCredentials` - List service-specific credentials
- `ResetServiceSpecificCredential` - Reset service-specific credentials

</details>

<details>
<summary><strong>Groups</strong></summary>

- `CreateGroup` - Create a new group
- `UpdateGroup` - Update group properties
- `DeleteGroup` - Delete a group
- `GetGroup` - Get group information
- `ListGroups` - List all groups
- `ListGroupsForUser` - List groups for a user
- `AddUserToGroup` - Add user to group
- `RemoveUserFromGroup` - Remove user from group
- `AttachGroupPolicy` - Attach policy to group
- `DetachGroupPolicy` - Detach policy from group
- `ListAttachedGroupPolicies` - List attached policies
- `PutGroupPolicy` - Put inline policy
- `GetGroupPolicy` - Get inline policy
- `ListGroupPolicies` - List inline policies
- `DeleteGroupPolicy` - Delete inline policy

</details>

<details>
<summary><strong>Roles</strong></summary>

- `CreateRole` - Create a new role
- `UpdateRole` - Update role properties
- `UpdateRoleDescription` - Update role description
- `GetRole` - Get role information
- `DeleteRole` - Delete a role
- `ListRoles` - List all roles
- `ListRoleTags` - List role tags
- `PutRolePolicy` - Put inline policy
- `GetRolePolicy` - Get inline policy
- `DeleteRolePolicy` - Delete inline policy
- `ListRolePolicies` - List inline policies
- `AttachRolePolicy` - Attach managed policy
- `DetachRolePolicy` - Detach managed policy
- `ListAttachedRolePolicies` - List attached policies
- `UpdateAssumeRolePolicy` - Update trust policy
- `CreateInstanceProfile` - Create instance profile
- `AddRoleToInstanceProfile` - Add role to instance profile
- `RemoveRoleFromInstanceProfile` - Remove role from instance profile
- `DeleteInstanceProfile` - Delete instance profile
- `GetInstanceProfile` - Get instance profile
- `ListInstanceProfiles` - List instance profiles
- `ListInstanceProfilesForRole` - List instance profiles for role

</details>

<details>
<summary><strong>Policies</strong></summary>

- `CreatePolicy` - Create managed policy
- `CreatePolicyVersion` - Create policy version
- `DeletePolicy` - Delete managed policy
- `DeletePolicyVersion` - Delete policy version
- `GetPolicy` - Get managed policy
- `GetPolicyVersion` - Get policy version
- `ListPolicies` - List managed policies
- `ListPolicyVersions` - List policy versions
- `ListEntitiesForPolicy` - List entities for policy
- `SetDefaultPolicyVersion` - Set default policy version
- `ListPoliciesGrantingServiceAccess` - List policies granting service access
- `PutUserPolicy` - Put user inline policy
- `GetUserPolicy` - Get user inline policy
- `DeleteUserPolicy` - Delete user inline policy
- `ListUserPolicies` - List user inline policies
- `AttachUserPolicy` - Attach managed policy to user
- `DetachUserPolicy` - Detach managed policy from user

</details>

<details>
<summary><strong>Permissions Boundaries</strong></summary>

- `PutUserPermissionsBoundary` - Set user permissions boundary
- `DeleteUserPermissionsBoundary` - Delete user permissions boundary
- `PutRolePermissionsBoundary` - Set role permissions boundary
- `DeleteRolePermissionsBoundary` - Delete role permissions boundary

</details>

<details>
<summary><strong>Policy Evaluation</strong></summary>

- `SimulateCustomPolicy` - Simulate custom policy
- `SimulatePrincipalPolicy` - Simulate principal policy
- `GetContextKeysForCustomPolicy` - Get context keys for custom policy
- `GetContextKeysForPrincipalPolicy` - Get context keys for principal policy
- `GenerateServiceLastAccessedDetails` - Generate service last accessed details
- `GetServiceLastAccessedDetails` - Get service last accessed details
- `GetServiceLastAccessedDetailsWithEntities` - Get service last accessed details with entities
- `GenerateOrganizationsAccessReport` - Generate organizations access report
- `GetOrganizationsAccessReport` - Get organizations access report

</details>

<details>
<summary><strong>Identity Providers</strong></summary>

**SAML Providers:**
- `CreateSAMLProvider` - Create SAML provider
- `UpdateSAMLProvider` - Update SAML provider
- `DeleteSAMLProvider` - Delete SAML provider
- `GetSAMLProvider` - Get SAML provider
- `ListSAMLProviders` - List SAML providers

**OIDC Providers:**
- `CreateOpenIDConnectProvider` - Create OIDC provider
- `UpdateOpenIDConnectProviderThumbprint` - Update OIDC provider thumbprint
- `DeleteOpenIDConnectProvider` - Delete OIDC provider
- `GetOpenIDConnectProvider` - Get OIDC provider
- `ListOpenIDConnectProviders` - List OIDC providers
- `TagOpenIDConnectProvider` - Tag OIDC provider
- `UntagOpenIDConnectProvider` - Untag OIDC provider

</details>

<details>
<summary><strong>Server Certificates</strong></summary>

- `UploadServerCertificate` - Upload server certificate
- `UpdateServerCertificate` - Update server certificate
- `DeleteServerCertificate` - Delete server certificate
- `GetServerCertificate` - Get server certificate
- `ListServerCertificates` - List server certificates

</details>

<details>
<summary><strong>Service Linked Roles</strong></summary>

- `CreateServiceLinkedRole` - Create service-linked role
- `DeleteServiceLinkedRole` - Delete service-linked role
- `GetServiceLinkedRoleDeletionStatus` - Get deletion status
- `ListRoles` - List roles (including service-linked)

</details>

<details>
<summary><strong>Tags</strong></summary>

- `TagUser` - Tag a user
- `UntagUser` - Untag a user
- `ListUserTags` - List user tags
- `TagRole` - Tag a role
- `UntagRole` - Untag a role
- `ListRoleTags` - List role tags
- `TagPolicy` - Tag a policy
- `UntagPolicy` - Untag a policy
- `ListPolicyTags` - List policy tags

</details>

<details>
<summary><strong>Reports</strong></summary>

- `GenerateCredentialReport` - Generate credential report
- `GetCredentialReport` - Get credential report
- `GetAccountSummary` - Get account summary
- `GetAccountPasswordPolicy` - Get password policy
- `UpdateAccountPasswordPolicy` - Update password policy
- `DeleteAccountPasswordPolicy` - Delete password policy
- `GenerateServiceLastAccessedDetails` - Generate service last accessed details
- `GetServiceLastAccessedDetails` - Get service last accessed details
- `GetServiceLastAccessedDetailsWithEntities` - Get service last accessed details with entities

</details>

<details>
<summary><strong>Signing Certificates</strong></summary>

- `UploadSigningCertificate` - Upload signing certificate
- `UpdateSigningCertificate` - Update signing certificate
- `DeleteSigningCertificate` - Delete signing certificate
- `ListSigningCertificates` - List signing certificates

</details>

### 🔑 STS Operations

<details>
<summary><strong>Temporary Credentials</strong></summary>

- `AssumeRole` - Assume a role
- `AssumeRoleWithSAML` - Assume role with SAML
- `AssumeRoleWithWebIdentity` - Assume role with web identity
- `AssumeRoleWithClientGrants` - Assume role with client grants
- `GetFederationToken` - Get federation token
- `GetSessionToken` - Get session token
- `DecodeAuthorizationMessage` - Decode authorization message
- `GetAccessKeyInfo` - Get access key information

</details>

<details>
<summary><strong>Identity Inspection</strong></summary>

- `GetCallerIdentity` - Get caller identity information

</details>

### 🏢 SSO Admin Operations

<details>
<summary><strong>Permission Sets</strong></summary>

- `CreatePermissionSet` - Create permission set
- `UpdatePermissionSet` - Update permission set
- `DeletePermissionSet` - Delete permission set
- `DescribePermissionSet` - Describe permission set
- `ListPermissionSets` - List permission sets
- `ListPermissionSetsProvisionedToAccount` - List provisioned permission sets
- `ListCustomerManagedPolicyReferencesInPermissionSet` - List customer managed policy references
- `AttachCustomerManagedPolicyReferenceToPermissionSet` - Attach customer managed policy reference
- `DetachCustomerManagedPolicyReferenceFromPermissionSet` - Detach customer managed policy reference
- `AttachManagedPolicyToPermissionSet` - Attach managed policy
- `DetachManagedPolicyFromPermissionSet` - Detach managed policy
- `ListManagedPoliciesInPermissionSet` - List managed policies
- `ProvisionPermissionSet` - Provision permission set
- `DescribePermissionSetProvisioningStatus` - Describe provisioning status
- `ListPermissionSetProvisioningStatus` - List provisioning status

</details>

<details>
<summary><strong>Assignments</strong></summary>

- `CreateAccountAssignment` - Create account assignment
- `DeleteAccountAssignment` - Delete account assignment
- `DescribeAccountAssignmentCreationStatus` - Describe creation status
- `DescribeAccountAssignmentDeletionStatus` - Describe deletion status
- `ListAccountAssignments` - List account assignments
- `ListAccountAssignmentCreationStatus` - List creation status
- `ListAccountAssignmentDeletionStatus` - List deletion status

</details>

<details>
<summary><strong>Instances</strong></summary>

- `ListInstances` - List SSO instances
- `DescribeInstanceAccessControlAttributeConfiguration` - Describe access control attributes
- `PutInstanceAccessControlAttributeConfiguration` - Put access control attributes
- `DeleteInstanceAccessControlAttributeConfiguration` - Delete access control attributes
- `ListTagsForResource` - List resource tags
- `TagResource` - Tag resource
- `UntagResource` - Untag resource

</details>

<details>
<summary><strong>Applications</strong></summary>

- `ListApplications` - List applications
- `DescribeApplicationAssignment` - Describe application assignment
- `CreateApplicationAssignment` - Create application assignment
- `DeleteApplicationAssignment` - Delete application assignment
- `ListApplicationAssignments` - List application assignments

</details>

<details>
<summary><strong>Trusted Sources</strong></summary>

- `CreateTrustedTokenIssuer` - Create trusted token issuer
- `DeleteTrustedTokenIssuer` - Delete trusted token issuer
- `UpdateTrustedTokenIssuer` - Update trusted token issuer
- `ListTrustedTokenIssuers` - List trusted token issuers
- `DescribeTrustedTokenIssuer` - Describe trusted token issuer

</details>

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ami = "0.1.0"
```

## Usage

### Basic Usage with Auto-Generated Account ID

```rust
use ami::{MemoryIamClient, MemoryStsClient, MemorySsoAdminClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see account ID generation
    env_logger::init();
    
    // Initialize AWS clients with auto-generated account ID
    let store = ami::create_memory_store();
    let account_id = ami::get_account_id_from_store(&store);
    println!("Using AWS account ID: {}", account_id);
    
    // Print AWS environment variables for export
    ami::print_aws_environment_variables(&store);
    
    let mut iam_client = MemoryIamClient::new(store);
    let mut sts_client = MemoryStsClient::new(store);
    let mut sso_client = MemorySsoAdminClient::new(store);
    
    // Get account ID from client
    let client_account_id = iam_client.account_id().await?;
    println!("Account ID from IAM client: {}", client_account_id);
    
    // Example: Create a user
    let user_request = ami::CreateUserRequest {
        user_name: "test-user".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let user = iam_client.create_user(user_request).await?;
    println!("Created user ARN: {}", user.data.unwrap().arn);
    
    // Example: Get caller identity
    let identity = sts_client.get_caller_identity().await?;
    println!("Current user: {}", identity.data.unwrap().arn);
    
    Ok(())
}
```

### Using Custom Account ID

```rust
use ami::{create_memory_store_with_account_id, MemoryIamClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a specific account ID
    let store = create_memory_store_with_account_id("123456789012".to_string());
    let mut iam_client = MemoryIamClient::new(store);
    
    // All ARNs will use the specified account ID
    let user_request = ami::CreateUserRequest {
        user_name: "my-user".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let user = iam_client.create_user(user_request).await?;
    println!("User ARN: {}", user.data.unwrap().arn);
    // Output: arn:aws:iam::123456789012:user/my-user
    
    Ok(())
}
```

### Account ID Management

AMI.rs automatically generates realistic 12-digit AWS account IDs for each instance. You can:

- **Auto-generate**: Use `create_memory_store()` for a random account ID
- **Custom ID**: Use `create_memory_store_with_account_id("123456789012")` for a specific ID
- **Retrieve ID**: Use `get_account_id_from_store(&store)` or `client.account_id().await?`
- **Logging**: Enable logging with `env_logger::init()` to see account ID generation

All ARNs (users, groups, roles, policies) will use the same account ID consistently across IAM, STS, and SSO Admin operations.

### AWS Environment Variables

AMI.rs provides AWS environment variables for compatibility with AWS CLI and other tools:

```rust
use ami::{create_memory_store, print_aws_environment_variables};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Enable logging
    
    let store = create_memory_store();
    
    // Print environment variables to console
    print_aws_environment_variables(&store);
    
    // Or get them programmatically
    let env_vars = store.aws_environment_variables();
    println!("AWS_ACCOUNT_ID: {}", env_vars["AWS_ACCOUNT_ID"]);
    
    Ok(())
}
```

**Console Output:**
```
INFO ami::store::in_memory: Generated AWS account ID: 847392847392
INFO ami::store::in_memory: AWS Environment Variables for export:
INFO ami::store::in_memory:   export AWS_ACCOUNT_ID=847392847392
INFO ami::store::in_memory:   export AWS_DEFAULT_REGION=us-east-1
INFO ami::store::in_memory:   export AWS_REGION=us-east-1
INFO ami::store::in_memory:   export AWS_PROFILE=default
INFO ami::store::in_memory: 
INFO ami::store::in_memory: To use with AWS CLI or other tools, run:
INFO ami::store::in_memory:   export AWS_ACCOUNT_ID=847392847392
INFO ami::store::in_memory:   export AWS_DEFAULT_REGION=us-east-1

AWS Environment Variables:
  export AWS_ACCOUNT_ID=847392847392
  export AWS_DEFAULT_REGION=us-east-1
  export AWS_REGION=us-east-1
  export AWS_PROFILE=default

To use with AWS CLI or other tools, run:
  export AWS_ACCOUNT_ID=847392847392
  export AWS_DEFAULT_REGION=us-east-1
```

**Export for AWS CLI:**
```bash
export AWS_ACCOUNT_ID=847392847392
export AWS_DEFAULT_REGION=us-east-1
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
