# Issue #001: Implement Policy Condition Keys

**Status**: 🔴 Open  
**Priority**: High  
**Type**: Feature Enhancement  
**Assignee**: TBD  
**Created**: 2025-10-29  
**Labels**: `enhancement`, `policy-engine`, `aws-compatibility`, `wami-feature`

## Summary

Implement comprehensive condition key support in the WAMI policy evaluation engine to enable fine-grained access control based on request context. This includes all AWS global condition keys plus extensive WAMI-specific extensions for multi-cloud and multi-tenant scenarios.

## Current State

- ✅ `PolicyStatement` has a `condition` field (as `Option<Value>` in `src/types.rs`)
- ❌ Condition evaluation not implemented (TODOs in `src/service/policies/evaluation.rs:79,133`)
- ❌ Context values not collected during requests
- ❌ No structured condition types or operators

## Problem Statement

Currently, WAMI policies can only control access based on Action and Resource. There's no way to:
- Restrict access based on IP address, time, or network conditions
- Enforce MFA requirements
- Implement tenant isolation via policy conditions
- Apply different rules based on authentication strength
- Enforce compliance requirements (data classification, geo-fencing, etc.)
- Control cross-cloud or cross-tenant access

## Proposed Solution

Implement a complete condition evaluation system with:

1. **Core Infrastructure** (2 weeks)
   - Condition evaluation module (`src/wami/policies/condition.rs`)
   - Structured condition types and operators
   - Request context collection framework
   - Integration with policy evaluation engine

2. **AWS Compatibility** (2 weeks)
   - All 40+ AWS global condition keys
   - AWS-compatible operators (String, Numeric, Date, IP, ARN, etc.)
   - Behavior matching AWS IAM policy simulator

3. **WAMI Extensions** (2 weeks)
   - Multi-cloud condition keys (70+ keys)
   - Multi-tenant isolation keys
   - Advanced security conditions
   - Custom operators (RegEx, SemVer, GeoDistance, etc.)

4. **Testing & Documentation** (2 weeks)
   - Comprehensive test coverage
   - AWS compatibility validation
   - User guides and examples
   - Migration documentation

## Operator Quick Reference

| Operator Type | Example | Use Case |
|--------------|---------|----------|
| `StringEquals` | `"aws:username": "alice"` | Exact string match |
| `StringLike` | `"aws:PrincipalArn": "arn:aws:iam::*:role/Admin*"` | Wildcard matching |
| `StringEqualsIfExists` | `"aws:PrincipalTag/Env": "prod"` | Optional tag check |
| `NumericLessThan` | `"aws:MultiFactorAuthAge": "3600"` | MFA freshness check |
| `DateGreaterThan` | `"aws:CurrentTime": "2025-01-01T00:00:00Z"` | Time-based access |
| `IpAddress` | `"aws:SourceIp": "203.0.113.0/24"` | IP allowlisting |
| `Bool` | `"aws:SecureTransport": "true"` | Require HTTPS |
| `ArnLike` | `"aws:SourceArn": "arn:aws:s3:::*"` | Resource type matching |
| `Null` | `"aws:TokenIssueTime": "false"` | Check if temporary creds |
| `ForAllValues:StringEquals` | `"aws:TagKeys": ["Env", "Owner"]` | Restrict allowed tags (AND) |
| `ForAnyValue:StringEquals` | `"aws:PrincipalTag/Role": ["Admin", "DevOps"]` | Role-based access (OR) |

## Detailed Condition Keys

### AWS Global Condition Keys (40+ keys)

#### Principal & Identity
- `aws:PrincipalArn`, `aws:PrincipalAccount`, `aws:PrincipalOrgID`, `aws:PrincipalType`
- `aws:username`, `aws:userId`, `aws:userid`
- `aws:PrincipalTag/${TagKey}`, `aws:PrincipalServiceName`

#### Authentication
- `aws:MultiFactorAuthPresent`, `aws:MultiFactorAuthAge`
- `aws:TokenIssueTime`, `aws:FederatedProvider`

#### Network & Transport
- `aws:SourceIp`, `aws:SourceVpc`, `aws:SourceVpce`, `aws:VpcSourceIp`
- `aws:SecureTransport`, `aws:TLSCipher`, `aws:TLSServerName`

#### Request Context
- `aws:RequestedRegion`, `aws:Referer`, `aws:UserAgent`
- `aws:RequestTag/${TagKey}`, `aws:TagKeys`
- `aws:ViaAWSService`, `aws:CalledVia`

#### Resource & Source
- `aws:SourceAccount`, `aws:SourceArn`, `aws:SourceOrgID`
- `aws:ResourceAccount`, `aws:ResourceOrgID`, `aws:ResourceTag/${TagKey}`

#### Time
- `aws:CurrentTime`, `aws:EpochTime`

### WAMI-Specific Condition Keys (100+ keys)

#### Multi-Cloud (10 keys)
- `wami:Provider`, `wami:ProviderRegion`, `wami:ProviderAccountId`
- `wami:CrossProviderRequest`, `wami:SourceProvider`, `wami:TargetProvider`
- `wami:ArnProvider`, `wami:ArnService`, `wami:ArnResourceType`

#### Multi-Tenant (15 keys)
- `wami:TenantId`, `wami:TenantName`, `wami:TenantTier`, `wami:TenantStatus`
- `wami:PrincipalTenantId`, `wami:ResourceTenantId`, `wami:CrossTenantRequest`
- `wami:TenantHierarchyPath`, `wami:ParentTenantId`, `wami:RootTenantId`
- `wami:TenantTag/${TagKey}`, `wami:TenantRegion`, `wami:TenantCreationDate`

#### Advanced Authentication (12 keys)
- `wami:AuthenticationMethod`, `wami:AuthenticationStrength`
- `wami:MultiFactorMethods`, `wami:BiometricAuthPresent`, `wami:HardwareTokenPresent`
- `wami:PasswordAge`, `wami:AccountAge`, `wami:SessionRiskScore`
- `wami:IdentitySource`, `wami:IdentityProvider`, `wami:AssumeRoleChain`, `wami:AssumeRoleDepth`

#### Network & Security (15 keys)
- `wami:SourceCountry`, `wami:SourceCity`, `wami:SourceASN`
- `wami:RequestProtocol`, `wami:RequestMethod`, `wami:ApiVersion`
- `wami:VpnDetected`, `wami:TorExitNode`, `wami:KnownBotUserAgent`
- `wami:ProxyPresent`, `wami:ClientVersion`

#### Data & Compliance (8 keys)
- `wami:ResourceDataClassification`, `wami:ResourceComplianceTags`
- `wami:DataResidencyRegion`, `wami:EncryptionRequired`, `wami:EncryptionAlgorithm`
- `wami:DataRetentionPeriod`

#### Rate Limiting (6 keys)
- `wami:RequestsPerMinute`, `wami:RequestsPerHour`, `wami:RequestsPerDay`
- `wami:QuotaRemaining`, `wami:BurstCapacityUsed`

#### Cost & Billing (5 keys)
- `wami:EstimatedCost`, `wami:BillingProject`, `wami:CostCenter`
- `wami:BudgetRemaining`, `wami:CostAllocationTag/${TagKey}`

#### Observability (5 keys)
- `wami:RequestId`, `wami:CorrelationId`, `wami:TraceId`
- `wami:DebugModeEnabled`, `wami:DryRunMode`

## Implementation Plan

### Phase 1: Core Infrastructure (Weeks 1-2)
```rust
// src/wami/policies/condition/mod.rs
pub mod operators;
pub mod keys;
pub mod context;
pub mod evaluator;
```

**Tasks**:
- [ ] Create `ConditionKey` enum with all keys
- [ ] Create `ConditionOperator` enum
- [ ] Create `ConditionContext` struct for request context
- [ ] Implement condition evaluator
- [ ] Update `PolicyStatement` structure
- [ ] Basic unit tests

### Phase 2: AWS Operators (Weeks 3-4)

**Tasks**:
- [ ] **String operators** (10 total):
  - StringEquals, StringNotEquals, StringLike, StringNotLike
  - StringEqualsIgnoreCase, StringNotEqualsIgnoreCase
  - StringEqualsIfExists, StringNotEqualsIfExists
  - StringLikeIfExists, StringNotLikeIfExists
- [ ] **Numeric operators** (12 total):
  - NumericEquals, NumericNotEquals
  - NumericLessThan, NumericLessThanEquals
  - NumericGreaterThan, NumericGreaterThanEquals
  - NumericEqualsIfExists, NumericNotEqualsIfExists
  - NumericLessThanIfExists, NumericLessThanEqualsIfExists
  - NumericGreaterThanIfExists, NumericGreaterThanEqualsIfExists
- [ ] **Date/time operators** (12 total):
  - DateEquals, DateNotEquals
  - DateLessThan, DateLessThanEquals
  - DateGreaterThan, DateGreaterThanEquals
  - DateEqualsIfExists, DateNotEqualsIfExists
  - DateLessThanIfExists, DateLessThanEqualsIfExists
  - DateGreaterThanIfExists, DateGreaterThanEqualsIfExists
- [ ] **IP address operators** (4 total):
  - IpAddress, NotIpAddress
  - IpAddressIfExists, NotIpAddressIfExists
- [ ] **ARN operators** (8 total):
  - ArnEquals, ArnNotEquals, ArnLike, ArnNotLike
  - ArnEqualsIfExists, ArnNotEqualsIfExists
  - ArnLikeIfExists, ArnNotLikeIfExists
- [ ] **Boolean operators** (2 total):
  - Bool, BoolIfExists
- [ ] **Binary operators** (2 total):
  - BinaryEquals, BinaryEqualsIfExists
- [ ] **Null operator** (2 total):
  - Null, NullIfExists
- [ ] **ForAllValues set operators** (10 total):
  - ForAllValues:StringEquals, ForAllValues:StringLike, ForAllValues:StringNotLike
  - ForAllValues:ArnEquals, ForAllValues:ArnLike
  - ForAllValues:NumericLessThan, ForAllValues:NumericGreaterThan
  - ForAllValues:IpAddress
  - ForAllValues:DateLessThan, ForAllValues:DateGreaterThan
- [ ] **ForAnyValue set operators** (10 total):
  - ForAnyValue:StringEquals, ForAnyValue:StringLike, ForAnyValue:StringNotLike
  - ForAnyValue:ArnEquals, ForAnyValue:ArnLike
  - ForAnyValue:NumericLessThan, ForAnyValue:NumericGreaterThan
  - ForAnyValue:IpAddress
  - ForAnyValue:DateLessThan, ForAnyValue:DateGreaterThan
- [ ] AWS compatibility tests

**Total AWS Operators**: ~72 operators

### Phase 3: AWS Condition Keys (Weeks 3-4)

**Tasks**:
- [ ] Principal & identity keys
- [ ] Authentication keys
- [ ] Network & transport keys
- [ ] Request context keys
- [ ] Resource keys
- [ ] Time keys
- [ ] Integration tests

### Phase 4: WAMI Extensions (Weeks 5-6)

**Tasks**:
- [ ] Multi-cloud condition keys
- [ ] Multi-tenant condition keys
- [ ] Advanced auth keys
- [ ] Network security keys
- [ ] Data & compliance keys
- [ ] Rate limiting keys
- [ ] Cost & billing keys
- [ ] Custom operators (RegEx, SemVer, GeoDistance, etc.)
- [ ] WAMI-specific tests

### Phase 5: Integration (Weeks 7-8)

**Tasks**:
- [ ] Integrate with policy evaluation engine
- [ ] Update `EvaluationService::evaluate_action()` to use conditions
- [ ] Context collection from requests
- [ ] Remove TODO comments from evaluation.rs
- [ ] End-to-end tests
- [ ] Performance benchmarks
- [ ] Security audit

### Phase 6: Documentation (Week 8)

**Tasks**:
- [ ] API reference for all condition keys
- [ ] User guide with examples
- [ ] Migration guide from AWS
- [ ] Best practices document
- [ ] Update EXAMPLES.md
- [ ] Update API_REFERENCE.md

## Examples

### Example 1: IP Restriction
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "s3:*",
    "Resource": "*",
    "Condition": {
      "IpAddress": {
        "aws:SourceIp": ["203.0.113.0/24", "198.51.100.0/24"]
      }
    }
  }]
}
```

### Example 2: MFA Requirement
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "iam:DeleteUser",
    "Resource": "*",
    "Condition": {
      "Bool": {
        "aws:MultiFactorAuthPresent": "true"
      },
      "NumericLessThan": {
        "aws:MultiFactorAuthAge": "3600"
      }
    }
  }]
}
```

### Example 3: Tenant Isolation
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "wami:*",
    "Resource": "*",
    "Condition": {
      "StringEquals": {
        "wami:TenantId": "${wami:PrincipalTenantId}",
        "wami:TenantStatus": "active"
      },
      "StringNotEquals": {
        "wami:CrossTenantRequest": "true"
      }
    }
  }]
}
```

### Example 4: Multi-Cloud Access Control
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
      }
    }
  }]
}
```

### Example 5: Advanced Security
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
        "wami:TorExitNode": "true"
      }
    }
  }]
}
```

### Example 6: Cost Control
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Deny",
    "Action": ["ec2:RunInstances", "rds:CreateDBInstance"],
    "Resource": "*",
    "Condition": {
      "NumericGreaterThan": {
        "wami:EstimatedCost": "1000"
      },
      "NumericLessThan": {
        "wami:BudgetRemaining": "500"
      }
    }
  }]
}
```

## Testing Strategy

1. **Unit Tests**: Each operator and condition key
2. **Integration Tests**: Full policy evaluation with conditions
3. **AWS Compatibility**: Validate against AWS IAM behavior
4. **Multi-Tenant**: Tenant isolation validation
5. **Performance**: Benchmark complex policies
6. **Security**: Condition bypass prevention

## Success Criteria

- [ ] All AWS global condition keys implemented
- [ ] All WAMI-specific condition keys implemented
- [ ] AWS compatibility verified with test suite
- [ ] Performance acceptable (< 10ms for typical policies)
- [ ] Zero security vulnerabilities in condition evaluation
- [ ] Complete documentation and examples
- [ ] Migration guide from AWS policies

## Dependencies

- No external dependencies required
- Internal dependencies on existing policy engine

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| AWS compatibility issues | High | Comprehensive testing against AWS IAM |
| Performance degradation | Medium | Caching, lazy evaluation, profiling |
| Security vulnerabilities | High | Security audit, fuzzing, penetration testing |
| Complexity explosion | Medium | Phased approach, clear abstractions |

## Related Issues

- N/A (First issue)

## Related Documentation

- `docs/CONDITION_KEYS_IMPLEMENTATION.md` - Full implementation details
- `src/types.rs:106-107` - Existing condition field
- `src/service/policies/evaluation.rs:79,133` - TODO comments

## References

- [AWS IAM Condition Keys](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_condition-keys.html)
- [AWS Condition Operators](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition_operators.html)
- [Azure ABAC Conditions](https://learn.microsoft.com/en-us/azure/role-based-access-control/conditions-format)
- [GCP IAM Conditions](https://cloud.google.com/iam/docs/conditions-overview)

---

**Estimated Effort**: 8 weeks (1 developer)  
**Estimated LOC**: ~3,000-5,000 lines of new code

