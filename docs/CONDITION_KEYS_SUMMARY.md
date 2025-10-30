# Policy Condition Keys - Executive Summary

## Overview

Add comprehensive policy condition support to WAMI, enabling fine-grained access control based on request context (IP address, time, MFA status, tenant, provider, etc.).

## The Problem

Currently, WAMI policies only support Action and Resource controls:
```json
{
  "Effect": "Allow",
  "Action": "s3:GetObject",
  "Resource": "arn:aws:s3:::mybucket/*"
}
```

This lacks contextual control for:
- IP-based restrictions
- MFA requirements
- Time-based access
- Tenant isolation
- Multi-cloud boundaries
- Compliance requirements

## The Solution

Implement AWS-compatible condition evaluation + WAMI extensions:

```json
{
  "Effect": "Allow",
  "Action": "s3:GetObject",
  "Resource": "*",
  "Condition": {
    "IpAddress": {
      "aws:SourceIp": "203.0.113.0/24"
    },
    "Bool": {
      "aws:SecureTransport": "true"
    },
    "StringEquals": {
      "wami:TenantId": "${wami:PrincipalTenantId}"
    }
  }
}
```

## Scope

### Condition Keys: 140+ Total

- **40+ AWS Global Keys**: PrincipalArn, SourceIp, MultiFactorAuthPresent, CurrentTime, etc.
- **100+ WAMI Extensions**: TenantId, Provider, SessionRiskScore, DataClassification, etc.

### Condition Operators: 91 Total

- **String** (16): StringEquals, StringLike, StringEqualsIfExists, ForAllValues, ForAnyValue, etc.
- **Numeric** (16): NumericEquals, NumericLessThan, NumericGreaterThan, IfExists variants, etc.
- **Date/Time** (16): DateEquals, DateLessThan, DateGreaterThan, IfExists variants, etc.
- **IP Address** (6): IpAddress, NotIpAddress, IfExists variants, set operators
- **ARN** (12): ArnEquals, ArnLike, IfExists variants, set operators
- **Boolean** (2): Bool, BoolIfExists
- **Binary** (2): BinaryEquals, BinaryEqualsIfExists
- **Null** (2): Null, NullIfExists
- **Set Operators** (20): ForAllValues:*, ForAnyValue:*
- **WAMI Extensions** (9): RegexMatch, SemanticVersion, GeoDistance, etc.

## Key Features

### 1. AWS Compatibility
✅ All AWS IAM condition keys and operators  
✅ Behavior matches AWS IAM policy simulator  
✅ Drop-in replacement for AWS policies  

### 2. Multi-Tenant Isolation
```json
{
  "Condition": {
    "StringEquals": {
      "wami:TenantId": "${wami:PrincipalTenantId}"
    },
    "StringEquals": {
      "wami:ResourceTenantId": "${wami:TenantId}"
    }
  }
}
```

### 3. Multi-Cloud Control
```json
{
  "Condition": {
    "StringLike": {
      "wami:Provider": ["AWS", "GCP"]
    },
    "Bool": {
      "wami:CrossProviderRequest": "false"
    }
  }
}
```

### 4. Advanced Security
```json
{
  "Condition": {
    "NumericGreaterThan": {
      "wami:AuthenticationStrength": "70"
    },
    "Bool": {
      "wami:TorExitNode": "false"
    }
  }
}
```

## Implementation Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. Core Infrastructure | 2 weeks | Condition evaluation engine |
| 2. AWS Operators | 2 weeks | All 72 AWS operators |
| 3. AWS Keys | 2 weeks | All 40+ AWS condition keys |
| 4. WAMI Extensions | 2 weeks | 100+ WAMI keys + 9 operators |
| 5. Integration | 2 weeks | Policy engine integration |
| 6. Documentation | 1 week | Guides and examples |
| **Total** | **~8 weeks** | **Production-ready** |

## Success Metrics

- ✅ All AWS condition operators implemented (72)
- ✅ All AWS global condition keys implemented (40+)
- ✅ All WAMI condition keys implemented (100+)
- ✅ AWS IAM compatibility verified (100% test pass rate)
- ✅ Performance < 10ms for typical policies
- ✅ Zero security vulnerabilities
- ✅ Complete documentation and examples

## Business Value

### Security
- Enforce MFA for sensitive operations
- Restrict access by IP/location
- Time-based access control
- Prevent unauthorized cross-tenant access

### Compliance
- Data residency enforcement
- Audit trail with contextual information
- PCI-DSS, HIPAA, GDPR compliance support
- Role-based access with conditions

### Multi-Tenancy
- Bulletproof tenant isolation via policy
- Hierarchical tenant structures
- Cross-tenant access controls
- Tenant-aware rate limiting

### Cost Control
- Budget-based access control
- Cost center restrictions
- Prevent expensive operations
- Usage quota enforcement

## Example Use Cases

### 1. Require MFA for IAM Operations
```json
{
  "Effect": "Allow",
  "Action": "iam:*",
  "Resource": "*",
  "Condition": {
    "Bool": {"aws:MultiFactorAuthPresent": "true"},
    "NumericLessThan": {"aws:MultiFactorAuthAge": "3600"}
  }
}
```

### 2. Restrict to Corporate Network
```json
{
  "Effect": "Allow",
  "Action": "*",
  "Resource": "*",
  "Condition": {
    "IpAddress": {
      "aws:SourceIp": ["10.0.0.0/8", "172.16.0.0/12"]
    }
  }
}
```

### 3. Enforce Tenant Isolation
```json
{
  "Effect": "Allow",
  "Action": "wami:*",
  "Resource": "*",
  "Condition": {
    "StringEquals": {
      "wami:TenantId": "${wami:PrincipalTenantId}",
      "wami:ResourceTenantId": "${wami:PrincipalTenantId}"
    }
  }
}
```

### 4. Prevent High-Risk Sessions
```json
{
  "Effect": "Deny",
  "Action": "*",
  "Resource": "*",
  "Condition": {
    "NumericGreaterThan": {"wami:SessionRiskScore": "80"},
    "Bool": {"wami:TorExitNode": "true"}
  }
}
```

### 5. Time-Based Access
```json
{
  "Effect": "Allow",
  "Action": "s3:*",
  "Resource": "*",
  "Condition": {
    "DateGreaterThan": {"aws:CurrentTime": "2025-01-01T00:00:00Z"},
    "DateLessThan": {"aws:CurrentTime": "2025-12-31T23:59:59Z"}
  }
}
```

### 6. Cost Budget Enforcement
```json
{
  "Effect": "Deny",
  "Action": ["ec2:RunInstances", "rds:CreateDBInstance"],
  "Resource": "*",
  "Condition": {
    "NumericLessThan": {"wami:BudgetRemaining": "100"}
  }
}
```

## Technical Highlights

### Architecture
```
Request → Context Builder → Condition Evaluator → Policy Decision
            ↓                     ↓
        [91 operators]      [140+ keys]
```

### Code Organization
```
src/wami/policies/condition/
├── mod.rs              # Public API
├── keys.rs             # Condition key definitions
├── operators.rs        # Operator implementations
├── context.rs          # Request context
├── evaluator.rs        # Evaluation engine
└── tests.rs            # Comprehensive tests
```

### Key Algorithms
- Wildcard matching with glob patterns
- CIDR IP range matching (IPv4/IPv6)
- ARN pattern matching
- Set operations (ForAllValues, ForAnyValue)
- Variable substitution
- Date/time comparison

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| AWS compatibility gaps | High | Extensive AWS IAM test suite |
| Performance degradation | Medium | Caching, lazy eval, profiling |
| Security vulnerabilities | High | Security audit, fuzzing |
| Complexity explosion | Medium | Phased implementation, clear abstractions |

## Dependencies

- **No external dependencies** - Pure Rust implementation
- **Internal dependencies**: Existing policy evaluation engine
- **Optional**: `ipnetwork` crate for IP CIDR matching
- **Optional**: `regex` crate for pattern matching

## Next Steps

1. **Review** this proposal with team
2. **Approve** scope and timeline
3. **Assign** developer(s) to project
4. **Create** detailed implementation plan
5. **Begin** Phase 1: Core Infrastructure

## Documentation

- 📋 [Issue #001](issues/ISSUE_001_CONDITION_KEYS.md) - Formal issue tracking
- 📖 [Implementation Guide](CONDITION_KEYS_IMPLEMENTATION.md) - Detailed technical specs
- 📚 [API Reference](API_REFERENCE.md) - API documentation

## Questions?

Contact the WAMI team or open a discussion in the project repository.

---

**Status**: 🔴 Open (Not Started)  
**Priority**: 🟠 High  
**Estimated Effort**: 8 weeks (1 FTE)  
**Estimated LOC**: ~4,000-6,000 lines

