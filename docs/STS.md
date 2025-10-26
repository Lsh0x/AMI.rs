# STS (Security Token Service) - Multicloud Design

This document describes how STS operations are modeled in a multicloud context.

- New ResourceTypes:
  - StsAssumedRole → assumed-role
  - StsFederatedUser → federated-user
  - StsSession → session
- WAMI ARNs for STS use service=sts, e.g. arn:wami:sts::123456789012:assumed-role/<name>
- Native identifiers are provided by the active CloudProvider implementation:
  - AWS: arn:aws:sts::ACCOUNT:assumed-role/<role>/<session>
  - GCP/Azure: placeholder formats via provider.generate_resource_identifier
- Session duration is validated by provider.validate_session_duration using provider.resource_limits
- ProviderConfig is recorded on each StsSession for cross-provider tracking.

Directory layout mirrors iam/user for each operation:
- src/sts/<operation>/{model.rs,requests.rs,builder.rs,operations.rs,mod.rs}
