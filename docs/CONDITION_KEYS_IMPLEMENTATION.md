# Policy Condition Keys Implementation

## Overview

This document tracks the implementation of condition keys for policy evaluation in WAMI. Condition keys allow fine-grained access control by evaluating contextual information about the request.

## Current State

- ✅ PolicyStatement has a `condition` field (as `Option<Value>`)
- ❌ Condition evaluation is not implemented (TODOs in `evaluation.rs` lines 79, 133)
- ❌ Context values are not collected during requests
- ❌ Missing structured condition types and operators

## Implementation Tasks

### 1. Core Infrastructure

- [ ] Create `src/wami/policies/condition.rs` module
- [ ] Define `ConditionKey` enum with all supported keys
- [ ] Define `ConditionOperator` enum (StringEquals, StringLike, NumericLessThan, DateGreaterThan, etc.)
- [ ] Create `ConditionContext` struct to hold request context
- [ ] Implement condition evaluation logic
- [ ] Update `PolicyStatement` to use structured conditions
- [ ] Integrate condition evaluation into policy engine

### 2. AWS Global Condition Keys

#### Principal Information
- [ ] `aws:PrincipalArn` - ARN of the principal making the request
- [ ] `aws:PrincipalAccount` - Account ID of the principal
- [ ] `aws:PrincipalOrgID` - AWS Organization ID of the principal
- [ ] `aws:PrincipalOrgPaths` - Organization path of the principal
- [ ] `aws:PrincipalType` - Type of principal (User, AssumedRole, Role, FederatedUser, Account, Service)
- [ ] `aws:PrincipalTag/${TagKey}` - Tag attached to the principal
- [ ] `aws:PrincipalServiceName` - Service principal name
- [ ] `aws:PrincipalServiceNamesList` - List of service principals
- [ ] `aws:PrincipalIsAWSService` - Whether the principal is an AWS service

#### User/Role Identity
- [ ] `aws:username` - Friendly name of the user (IAM user only)
- [ ] `aws:userid` - Unique identifier of the user
- [ ] `aws:userId` - Alternative form of userid (both should be supported)

#### Authentication & Session
- [ ] `aws:MultiFactorAuthPresent` - Whether MFA was used (true/false)
- [ ] `aws:MultiFactorAuthAge` - Seconds since MFA authentication
- [ ] `aws:TokenIssueTime` - Date/time when temporary credentials were issued
- [ ] `aws:SecureTransport` - Whether the request was sent using SSL/TLS (true/false)
- [ ] `aws:FederatedProvider` - Identity provider for federated users (e.g., cognito-identity.amazonaws.com)

#### Network & Transport
- [ ] `aws:SourceIp` - IP address of the requester
- [ ] `aws:SourceVpc` - VPC ID from which the request was made
- [ ] `aws:SourceVpce` - VPC endpoint ID from which the request was made
- [ ] `aws:VpcSourceIp` - IP address from VPC endpoint
- [ ] `aws:TLSCipher` - TLS cipher suite used
- [ ] `aws:TLSServerName` - TLS server name indication (SNI)

#### Request Source & Delegation
- [ ] `aws:ViaAWSService` - Whether the request was made by an AWS service
- [ ] `aws:CalledVia` - Service(s) through which the request was made (chain)
- [ ] `aws:CalledViaFirst` - First service in the chain
- [ ] `aws:CalledViaLast` - Last service in the chain
- [ ] `aws:SourceAccount` - Account ID that owns the source resource
- [ ] `aws:SourceArn` - ARN of the source resource
- [ ] `aws:SourceOrgID` - Organization ID that owns the source resource
- [ ] `aws:SourceOrgPaths` - Organization path of the source account

#### Resource Information
- [ ] `aws:ResourceAccount` - Account ID that owns the requested resource
- [ ] `aws:ResourceOrgID` - Organization ID that owns the requested resource
- [ ] `aws:ResourceOrgPaths` - Organization path of the resource owner
- [ ] `aws:ResourceTag/${TagKey}` - Tag attached to the resource

#### Time & Date
- [ ] `aws:CurrentTime` - Current date and time (ISO 8601 format)
- [ ] `aws:EpochTime` - Current time in Unix epoch format
- [ ] `aws:TokenIssueTime` - Date/time when credentials were issued

#### Request Context
- [ ] `aws:RequestedRegion` - AWS region to which the request was made
- [ ] `aws:Referer` - HTTP referer header value
- [ ] `aws:UserAgent` - HTTP user agent header value
- [ ] `aws:RequestTag/${TagKey}` - Tag passed in the request
- [ ] `aws:TagKeys` - List of tag keys in the request

#### Special Keys
- [ ] `aws:PrincipalIsRoot` - Whether the principal is the root account
- [ ] `aws:PrincipalSessionName` - Session name for assumed roles
- [ ] `aws:RequestedService` - Service being requested

### 3. WAMI-Specific Condition Keys

#### Multi-Cloud Context
- [ ] `wami:Provider` - Cloud provider (AWS, Azure, GCP, Custom)
- [ ] `wami:ProviderRegion` - Region in provider-specific format
- [ ] `wami:ProviderAccountId` - Account/subscription ID in provider format
- [ ] `wami:CrossProviderRequest` - Whether request crosses provider boundaries
- [ ] `wami:SourceProvider` - Source resource's cloud provider
- [ ] `wami:TargetProvider` - Target resource's cloud provider
- [ ] `wami:ProviderServiceVersion` - Version of the provider service

#### WAMI ARN Context
- [ ] `wami:Arn` - Normalized WAMI ARN
- [ ] `wami:ArnProvider` - Provider component of WAMI ARN
- [ ] `wami:ArnService` - Service component of WAMI ARN
- [ ] `wami:ArnResourceType` - Resource type from WAMI ARN
- [ ] `wami:SourceWamiArn` - WAMI ARN of source resource
- [ ] `wami:ResourceWamiArn` - WAMI ARN of target resource

#### Multi-Tenant Context
- [ ] `wami:TenantId` - Current tenant identifier
- [ ] `wami:TenantName` - Friendly name of the tenant
- [ ] `wami:TenantTier` - Tenant tier/plan (free, premium, enterprise, etc.)
- [ ] `wami:TenantRegion` - Primary region of the tenant
- [ ] `wami:TenantCreationDate` - Date when tenant was created
- [ ] `wami:TenantStatus` - Tenant status (active, suspended, trial, etc.)
- [ ] `wami:TenantTag/${TagKey}` - Tag attached to the tenant
- [ ] `wami:PrincipalTenantId` - Tenant ID of the principal
- [ ] `wami:ResourceTenantId` - Tenant ID of the resource
- [ ] `wami:CrossTenantRequest` - Whether request crosses tenant boundaries
- [ ] `wami:TenantHierarchyPath` - Path in tenant hierarchy (for nested tenants)
- [ ] `wami:ParentTenantId` - Parent tenant ID in hierarchy
- [ ] `wami:RootTenantId` - Root tenant ID in hierarchy

#### Identity Hierarchy
- [ ] `wami:IdentitySource` - Source of identity (native, federated, sso, service)
- [ ] `wami:IdentityProvider` - Identity provider name/ARN
- [ ] `wami:IdentityProviderType` - Type of IdP (SAML, OIDC, OAuth, etc.)
- [ ] `wami:FederationSessionDuration` - Duration of federated session
- [ ] `wami:AssumeRoleChain` - Full chain of assumed roles
- [ ] `wami:AssumeRoleDepth` - Number of role assumptions in chain
- [ ] `wami:OriginalPrincipalArn` - Original principal before role assumption

#### Advanced Authentication
- [ ] `wami:AuthenticationMethod` - Method used (password, api_key, certificate, token, etc.)
- [ ] `wami:AuthenticationStrength` - Numeric strength score (0-100)
- [ ] `wami:MultiFactorMethods` - List of MFA methods used
- [ ] `wami:BiometricAuthPresent` - Whether biometric auth was used
- [ ] `wami:HardwareTokenPresent` - Whether hardware token was used
- [ ] `wami:PasswordLastChanged` - Date when password was last changed
- [ ] `wami:PasswordAge` - Age of password in days
- [ ] `wami:AccountAge` - Age of account in days
- [ ] `wami:LastActivityTime` - Time of last activity by this principal
- [ ] `wami:SessionRiskScore` - Computed risk score for the session

#### Network & Security Context
- [ ] `wami:SourceCountry` - Country code of source IP (GeoIP)
- [ ] `wami:SourceCity` - City of source IP
- [ ] `wami:SourceASN` - Autonomous System Number
- [ ] `wami:RequestProtocol` - Protocol used (HTTP, gRPC, WebSocket, etc.)
- [ ] `wami:RequestMethod` - HTTP method (GET, POST, etc.)
- [ ] `wami:RequestPath` - Request path/endpoint
- [ ] `wami:ApiVersion` - API version being called
- [ ] `wami:ClientVersion` - Version of client SDK/tool
- [ ] `wami:ProxyPresent` - Whether request went through proxy
- [ ] `wami:VpnDetected` - Whether VPN usage detected
- [ ] `wami:TorExitNode` - Whether request from Tor exit node
- [ ] `wami:KnownBotUserAgent` - Whether user agent matches known bots

#### Data Classification & Compliance
- [ ] `wami:ResourceDataClassification` - Data classification level (public, internal, confidential, restricted)
- [ ] `wami:ResourceComplianceTags` - Compliance frameworks (HIPAA, PCI-DSS, GDPR, etc.)
- [ ] `wami:DataResidencyRegion` - Required data residency region
- [ ] `wami:EncryptionRequired` - Whether encryption is required
- [ ] `wami:EncryptionAlgorithm` - Encryption algorithm in use
- [ ] `wami:DataRetentionPeriod` - Data retention period in days

#### Rate Limiting & Quotas
- [ ] `wami:RequestsPerMinute` - Current request rate
- [ ] `wami:RequestsPerHour` - Request count in current hour
- [ ] `wami:RequestsPerDay` - Request count in current day
- [ ] `wami:QuotaRemaining` - Remaining quota for operation
- [ ] `wami:QuotaResetTime` - Time when quota resets
- [ ] `wami:BurstCapacityUsed` - Percentage of burst capacity used

#### Cost & Billing
- [ ] `wami:EstimatedCost` - Estimated cost of the operation
- [ ] `wami:BillingProject` - Billing project/cost center
- [ ] `wami:CostCenter` - Cost center tag
- [ ] `wami:BudgetRemaining` - Remaining budget for the period
- [ ] `wami:CostAllocationTag/${TagKey}` - Cost allocation tag

#### Service-Specific Context
- [ ] `wami:StsSessionDuration` - Requested STS session duration
- [ ] `wami:StsExternalId` - External ID for STS assume role
- [ ] `wami:SsoInstanceArn` - SSO instance ARN
- [ ] `wami:SsoPermissionSetArn` - SSO permission set ARN
- [ ] `wami:SsoApplicationArn` - SSO application ARN

#### Observability & Debugging
- [ ] `wami:RequestId` - Unique request identifier
- [ ] `wami:CorrelationId` - Correlation ID for distributed tracing
- [ ] `wami:TraceId` - Distributed trace ID
- [ ] `wami:DebugModeEnabled` - Whether debug mode is enabled
- [ ] `wami:DryRunMode` - Whether request is a dry run

### 4. Condition Operators

#### String Comparison Operators
- [ ] `StringEquals` - Case-sensitive exact match
- [ ] `StringNotEquals` - Case-sensitive not equal
- [ ] `StringEqualsIfExists` - StringEquals, but skip if key doesn't exist
- [ ] `StringNotEqualsIfExists` - StringNotEquals, but skip if key doesn't exist
- [ ] `StringLike` - Case-sensitive wildcard match (* and ?)
- [ ] `StringNotLike` - Case-sensitive wildcard not match
- [ ] `StringLikeIfExists` - StringLike, but skip if key doesn't exist
- [ ] `StringNotLikeIfExists` - StringNotLike, but skip if key doesn't exist
- [ ] `StringEqualsIgnoreCase` - Case-insensitive exact match (AWS extension)
- [ ] `StringNotEqualsIgnoreCase` - Case-insensitive not equal (AWS extension)

#### Numeric Comparison Operators
- [ ] `NumericEquals` - Numeric equality
- [ ] `NumericNotEquals` - Numeric inequality
- [ ] `NumericLessThan` - Less than
- [ ] `NumericLessThanEquals` - Less than or equal
- [ ] `NumericGreaterThan` - Greater than
- [ ] `NumericGreaterThanEquals` - Greater than or equal
- [ ] `NumericEqualsIfExists` - NumericEquals, but skip if key doesn't exist
- [ ] `NumericNotEqualsIfExists` - NumericNotEquals, but skip if key doesn't exist
- [ ] `NumericLessThanIfExists` - NumericLessThan, but skip if key doesn't exist
- [ ] `NumericLessThanEqualsIfExists` - NumericLessThanEquals, but skip if key doesn't exist
- [ ] `NumericGreaterThanIfExists` - NumericGreaterThan, but skip if key doesn't exist
- [ ] `NumericGreaterThanEqualsIfExists` - NumericGreaterThanEquals, but skip if key doesn't exist

#### Boolean Operators
- [ ] `Bool` - Boolean match (true/false)
- [ ] `BoolIfExists` - Bool, but skip if key doesn't exist

#### Date/Time Operators
- [ ] `DateEquals` - Date equality
- [ ] `DateNotEquals` - Date inequality
- [ ] `DateLessThan` - Before date
- [ ] `DateLessThanEquals` - Before or on date
- [ ] `DateGreaterThan` - After date
- [ ] `DateGreaterThanEquals` - After or on date
- [ ] `DateEqualsIfExists` - DateEquals, but skip if key doesn't exist
- [ ] `DateNotEqualsIfExists` - DateNotEquals, but skip if key doesn't exist
- [ ] `DateLessThanIfExists` - DateLessThan, but skip if key doesn't exist
- [ ] `DateLessThanEqualsIfExists` - DateLessThanEquals, but skip if key doesn't exist
- [ ] `DateGreaterThanIfExists` - DateGreaterThan, but skip if key doesn't exist
- [ ] `DateGreaterThanEqualsIfExists` - DateGreaterThanEquals, but skip if key doesn't exist

#### IP Address Operators
- [ ] `IpAddress` - IP in CIDR range
- [ ] `NotIpAddress` - IP not in CIDR range
- [ ] `IpAddressIfExists` - IpAddress, but skip if key doesn't exist
- [ ] `NotIpAddressIfExists` - NotIpAddress, but skip if key doesn't exist

#### Binary Comparison Operators
- [ ] `BinaryEquals` - Binary data equality
- [ ] `BinaryEqualsIfExists` - BinaryEquals, but skip if key doesn't exist

#### ARN/Pattern Matching Operators
- [ ] `ArnEquals` - ARN exact match
- [ ] `ArnNotEquals` - ARN not equal
- [ ] `ArnLike` - ARN wildcard match
- [ ] `ArnNotLike` - ARN wildcard not match
- [ ] `ArnEqualsIfExists` - ArnEquals, but skip if key doesn't exist
- [ ] `ArnNotEqualsIfExists` - ArnNotEquals, but skip if key doesn't exist
- [ ] `ArnLikeIfExists` - ArnLike, but skip if key doesn't exist
- [ ] `ArnNotLikeIfExists` - ArnNotLike, but skip if key doesn't exist

#### Null Check Operator
- [ ] `Null` - Check if key exists (true) or doesn't exist (false)
- [ ] `NullIfExists` - Null check with existence handling

#### Set Operators - ForAllValues (All values must match)
- [ ] `ForAllValues:StringEquals` - All request values match (AND logic)
- [ ] `ForAllValues:StringLike` - All request values match pattern
- [ ] `ForAllValues:StringNotLike` - All request values don't match pattern
- [ ] `ForAllValues:ArnEquals` - All ARNs match exactly
- [ ] `ForAllValues:ArnLike` - All ARNs match pattern
- [ ] `ForAllValues:NumericLessThan` - All numbers less than threshold
- [ ] `ForAllValues:NumericGreaterThan` - All numbers greater than threshold
- [ ] `ForAllValues:IpAddress` - All IPs in CIDR range
- [ ] `ForAllValues:DateLessThan` - All dates before threshold
- [ ] `ForAllValues:DateGreaterThan` - All dates after threshold

#### Set Operators - ForAnyValue (At least one value must match)
- [ ] `ForAnyValue:StringEquals` - Any request value matches (OR logic)
- [ ] `ForAnyValue:StringLike` - Any request value matches pattern
- [ ] `ForAnyValue:StringNotLike` - Any request value doesn't match pattern
- [ ] `ForAnyValue:ArnEquals` - Any ARN matches exactly
- [ ] `ForAnyValue:ArnLike` - Any ARN matches pattern
- [ ] `ForAnyValue:NumericLessThan` - Any number less than threshold
- [ ] `ForAnyValue:NumericGreaterThan` - Any number greater than threshold
- [ ] `ForAnyValue:IpAddress` - Any IP in CIDR range
- [ ] `ForAnyValue:DateLessThan` - Any date before threshold
- [ ] `ForAnyValue:DateGreaterThan` - Any date after threshold

### 5. Additional WAMI Operators

- [ ] `RegexMatch` - Regular expression matching
- [ ] `RegexNotMatch` - Regular expression not matching
- [ ] `SemanticVersionEquals` - Semantic version equality
- [ ] `SemanticVersionLessThan` - Semantic version comparison
- [ ] `SemanticVersionGreaterThan` - Semantic version comparison
- [ ] `JsonPathEquals` - JSONPath query matching
- [ ] `GeoDistance` - Geographic distance calculation
- [ ] `GeoWithinRegion` - Geographic region containment
- [ ] `HashEquals` - Hash comparison for sensitive values

### Operator Summary

**Total Operators**: ~82 AWS operators + 9 WAMI extensions = **91 total operators**

#### Understanding Operator Variants

**IfExists Variants** (~30 operators):
- All comparison operators have an `IfExists` variant
- Behavior: Returns `true` if the condition key is **not present** in the request context
- Use case: Make conditions optional - only check if the key exists
- Example: `StringEqualsIfExists` allows the condition to pass if `aws:PrincipalTag/Department` doesn't exist

**Set Operators** (20 operators):
- `ForAllValues:*` - ALL values in the request set must satisfy the condition (AND logic)
- `ForAnyValue:*` - AT LEAST ONE value in the request set must satisfy the condition (OR logic)
- Use case: When request context contains multiple values (e.g., multiple tags)
- Example: `ForAnyValue:StringEquals` on `aws:TagKeys` checks if any of the provided tags match

#### Operator Categories

| Category | Base Operators | IfExists Variants | Set Operators | Total |
|----------|----------------|-------------------|---------------|-------|
| String   | 6              | 4                 | 6             | 16    |
| Numeric  | 6              | 6                 | 4             | 16    |
| Date     | 6              | 6                 | 4             | 16    |
| IP       | 2              | 2                 | 2             | 6     |
| ARN      | 4              | 4                 | 4             | 12    |
| Boolean  | 1              | 1                 | 0             | 2     |
| Binary   | 1              | 1                 | 0             | 2     |
| Null     | 1              | 1                 | 0             | 2     |
| **AWS Total** | **27**     | **25**            | **20**        | **72** |
| WAMI Extensions | 9       | -                 | -             | 9     |
| **Grand Total** | **36**  | **25**            | **20**        | **91** |

## Implementation Approach

### Phase 1: Core Infrastructure (Week 1-2)
1. Create condition evaluation module
2. Define data structures for conditions and context
3. Implement basic condition operators (String, Numeric, Bool)
4. Add context collection to request processing

### Phase 2: AWS Compatibility (Week 3-4)
1. Implement all AWS global condition keys
2. Implement AWS-compatible operators
3. Add comprehensive test coverage
4. Validate against AWS IAM policy simulator behavior

### Phase 3: WAMI Extensions (Week 5-6)
1. Implement multi-cloud condition keys
2. Implement multi-tenant condition keys
3. Add WAMI-specific operators
4. Create migration guide from AWS policies

### Phase 4: Advanced Features (Week 7-8)
1. Implement condition key variables and substitution
2. Add policy condition validation
3. Create condition debugging tools
4. Performance optimization for complex conditions

## Example Usage

### AWS-Style Condition
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "s3:GetObject",
    "Resource": "arn:aws:s3:::mybucket/*",
    "Condition": {
      "IpAddress": {
        "aws:SourceIp": "203.0.113.0/24"
      },
      "Bool": {
        "aws:SecureTransport": "true"
      }
    }
  }]
}
```

### WAMI Multi-Tenant Condition
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "wami:*",
    "Resource": "*",
    "Condition": {
      "StringEquals": {
        "wami:TenantId": "${wami:PrincipalTenantId}"
      },
      "StringEquals": {
        "wami:TenantStatus": "active"
      },
      "NumericGreaterThan": {
        "wami:AuthenticationStrength": "70"
      }
    }
  }]
}
```

### WAMI Multi-Cloud Condition
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["s3:GetObject", "storage.objects.get"],
    "Resource": "*",
    "Condition": {
      "StringLike": {
        "wami:Provider": ["AWS", "GCP"]
      },
      "StringEquals": {
        "wami:ResourceDataClassification": "public"
      },
      "Bool": {
        "wami:SecureTransport": "true"
      }
    }
  }]
}
```

### WAMI Advanced Security Condition
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Deny",
    "Action": "*",
    "Resource": "*",
    "Condition": {
      "NumericGreaterThan": {
        "wami:SessionRiskScore": "80"
      },
      "Bool": {
        "wami:VpnDetected": "true",
        "wami:TorExitNode": "true"
      },
      "StringNotEquals": {
        "wami:SourceCountry": ["US", "CA", "GB"]
      }
    }
  }]
}
```

### Using IfExists Operators
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "s3:*",
    "Resource": "*",
    "Condition": {
      "StringEqualsIfExists": {
        "aws:PrincipalTag/Department": "Engineering"
      },
      "NumericLessThanIfExists": {
        "wami:SessionRiskScore": "50"
      }
    }
  }]
}
```
**Behavior**: 
- Allows access if the principal has no `Department` tag OR has `Department=Engineering`
- Allows access if risk score is not available OR is less than 50
- Useful for graceful handling of optional context values

### Using ForAllValues Set Operator
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "s3:PutObject",
    "Resource": "arn:aws:s3:::secure-bucket/*",
    "Condition": {
      "ForAllValues:StringEquals": {
        "aws:TagKeys": ["Project", "Owner", "CostCenter"]
      }
    }
  }]
}
```
**Behavior**:
- ALL tags provided in the request must be in the allowed list
- Prevents users from adding unauthorized tags
- Empty tag set passes (special case)

### Using ForAnyValue Set Operator
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "secretsmanager:GetSecretValue",
    "Resource": "*",
    "Condition": {
      "ForAnyValue:StringEquals": {
        "aws:PrincipalTag/Role": ["Admin", "DevOps", "SecurityEngineer"]
      }
    }
  }]
}
```
**Behavior**:
- AT LEAST ONE of the principal's Role tags must match the list
- Useful for OR logic across multiple values
- If principal has no Role tags, condition fails

### Combining Multiple Operators
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "iam:*",
    "Resource": "*",
    "Condition": {
      "Bool": {
        "aws:MultiFactorAuthPresent": "true"
      },
      "NumericLessThan": {
        "aws:MultiFactorAuthAge": "3600"
      },
      "IpAddress": {
        "aws:SourceIp": ["10.0.0.0/8", "172.16.0.0/12"]
      },
      "StringEqualsIfExists": {
        "wami:TenantId": "${wami:PrincipalTenantId}"
      },
      "ForAllValues:StringLike": {
        "aws:TagKeys": ["Env:*", "Team:*", "Cost:*"]
      }
    }
  }]
}
```
**Behavior**: ALL conditions must be satisfied (AND logic across condition blocks)

## Testing Requirements

1. **Unit Tests**: Each condition operator with various inputs
2. **Integration Tests**: Full policy evaluation with conditions
3. **AWS Compatibility Tests**: Validate against AWS IAM behavior
4. **Multi-Tenant Tests**: Tenant isolation validation
5. **Performance Tests**: Evaluate complex policies with many conditions
6. **Security Tests**: Ensure condition bypass is impossible

## Documentation Requirements

1. **API Reference**: Document all condition keys and operators
2. **User Guide**: How to write policies with conditions
3. **Migration Guide**: Convert AWS policies to WAMI
4. **Best Practices**: Security and performance recommendations
5. **Examples**: Common use cases and patterns

## Edge Cases and Important Notes

### IfExists Operator Behavior

```rust
// Pseudo-code for IfExists logic
fn evaluate_if_exists(key: &str, expected: Value, context: &Context) -> bool {
    match context.get(key) {
        None => true,  // Key doesn't exist -> condition passes
        Some(actual) => actual == expected  // Key exists -> normal comparison
    }
}
```

**Critical Rule**: `IfExists` returns `true` when the key is absent from context

### Set Operator Special Cases

#### ForAllValues with Empty Sets
```json
{
  "ForAllValues:StringEquals": {
    "aws:TagKeys": ["Project", "Owner"]
  }
}
```
- If request has NO tags → condition **PASSES** (vacuous truth)
- AWS behavior: Empty set satisfies "all values" condition
- This can be a security issue - use with care!

#### ForAnyValue with Empty Sets
```json
{
  "ForAnyValue:StringEquals": {
    "aws:PrincipalTag/Role": ["Admin"]
  }
}
```
- If principal has NO Role tags → condition **FAILS**
- AWS behavior: Can't find "any value" to match
- More secure default behavior

#### Combining ForAllValues and ForAnyValue
```json
{
  "ForAllValues:StringEquals": {
    "aws:TagKeys": ["Env", "Team", "Owner"]
  },
  "ForAnyValue:StringEquals": {
    "aws:TagKeys": ["Env", "Team", "Owner"]
  }
}
```
- `ForAllValues`: All request tags must be in the list (restricts to whitelist)
- `ForAnyValue`: At least one request tag must be in the list (requires minimum tags)
- Combined: Tags must be from whitelist AND at least one must be present

### Null Operator Behavior

```json
{
  "Null": {
    "aws:TokenIssueTime": "true"
  }
}
```
- `"true"`: Condition passes if key does NOT exist or is null
- `"false"`: Condition passes if key EXISTS and is not null
- Counter-intuitive: `"true"` means "is null/absent"

### Variable Substitution

Some condition values support policy variables:
```json
{
  "StringEquals": {
    "s3:prefix": "${aws:username}/*"
  }
}
```

Supported variables:
- `${aws:username}` - Current user name
- `${aws:userid}` - Current user ID
- `${aws:PrincipalTag/TagKey}` - Principal's tag value
- `${aws:CurrentTime}` - Current timestamp
- WAMI extensions:
  - `${wami:TenantId}` - Current tenant
  - `${wami:Provider}` - Current provider

### Wildcard Pattern Matching

String and ARN operators support wildcards:
- `*` - Matches any sequence of characters
- `?` - Matches exactly one character

```json
{
  "StringLike": {
    "aws:PrincipalArn": "arn:aws:iam::123456789012:role/Admin*"
  }
}
```

**Important**: 
- Wildcards only work with `*Like` operators, not `*Equals`
- Case-sensitive unless using `IgnoreCase` variant
- ARN wildcards must respect ARN structure

### IP Address CIDR Notation

```json
{
  "IpAddress": {
    "aws:SourceIp": ["203.0.113.0/24", "2001:db8::/32"]
  }
}
```
- Supports both IPv4 and IPv6
- CIDR notation required (even for single IPs: `203.0.113.42/32`)
- Multiple ranges can be specified in array

### Date/Time Format

AWS uses ISO 8601 format:
```json
{
  "DateGreaterThan": {
    "aws:CurrentTime": "2025-01-01T00:00:00Z"
  }
}
```
- Format: `YYYY-MM-DDThh:mm:ssZ`
- Always UTC (Z suffix)
- Epoch time alternative: numeric seconds since 1970-01-01

### Case Sensitivity Rules

| Operator | Case Sensitive | Notes |
|----------|----------------|-------|
| `StringEquals` | Yes | Exact match |
| `StringEqualsIgnoreCase` | No | AWS extension |
| `StringLike` | Yes | Pattern match |
| `ArnEquals` | Yes | ARN comparison |
| `ArnLike` | Yes | ARN pattern match |

### Condition Evaluation Order

1. **Within a condition block**: All operators are AND-ed
2. **Multiple condition blocks**: AND-ed together
3. **Multiple values in array**: OR-ed together

```json
{
  "Condition": {
    "StringEquals": {
      "aws:RequestedRegion": ["us-east-1", "us-west-2"]  // OR
    },
    "IpAddress": {
      "aws:SourceIp": "10.0.0.0/8"
    }
    // StringEquals AND IpAddress
  }
}
```

### Multi-Valued Context Keys

Some context keys naturally contain multiple values:
- `aws:TagKeys` - All tag keys in request
- `aws:PrincipalTag/*` - All principal tags
- `wami:MultiFactorMethods` - List of MFA methods used

Best practice: Use `ForAllValues` or `ForAnyValue` with multi-valued keys

## Performance Considerations

1. **Caching**: Cache condition evaluation results where appropriate
2. **Short-circuit**: Evaluate cheapest conditions first
3. **Indexing**: Index condition keys for fast lookup
4. **Lazy Evaluation**: Only compute context values when needed
5. **Parallel Evaluation**: Evaluate independent conditions in parallel
6. **Context Pre-computation**: Compute expensive context values once per request
7. **Wildcard Optimization**: Use trie/prefix trees for wildcard matching

## Security Considerations

1. **Context Validation**: Validate all context values from trusted sources
2. **Injection Prevention**: Prevent condition key injection attacks
3. **Information Leakage**: Ensure conditions don't leak sensitive info
4. **Bypass Protection**: Ensure conditions can't be bypassed
5. **Audit Logging**: Log all condition evaluations for audit

## Related Files

- `src/types.rs` - PolicyStatement already has `condition: Option<Value>`
- `src/service/policies/evaluation.rs` - TODO comments on lines 79, 133
- `src/wami/policies/evaluation/` - Evaluation request/response types
- Future: `src/wami/policies/condition.rs` - New module for implementation

## References

- [AWS IAM Policy Elements: Condition](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html)
- [AWS Global Condition Context Keys](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_condition-keys.html)
- [Azure ABAC Conditions](https://learn.microsoft.com/en-us/azure/role-based-access-control/conditions-format)
- [GCP IAM Conditions](https://cloud.google.com/iam/docs/conditions-overview)

