# Changelog

All notable changes to this project will be documented in this file.

## [0.10.0] - 2025-10-30

### 🚀 Features

- Implement permissions boundaries (Issue #22)

## [0.9.0] - 2025-10-30

### 🚀 Features

- Add multicloud provider infrastructure (Phase 1)
- Integrate provider system into Store traits (Phase 2)
- Refactor IAM modules to use CloudProvider (Phase 3a)
- Refactor roles.rs to use CloudProvider (Phase 3b)
- Refactor policies.rs to use CloudProvider (Phase 3c)
- Refactor server_certificates.rs to use CloudProvider (Phase 3d)
- **multicloud:** Complete Phase 3 - refactor remaining IAM modules to use CloudProvider
- **multicloud:** Add comprehensive provider tests and multicloud documentation (Phase 4)
- Add WAMI ARN and provider tracking to all resources
- Implement hierarchical multi-tenant architecture
- Implement hierarchical multi-tenant architecture
- Refactor tenant authorization to use IAM policy evaluation
- Implement secure ARN-centric architecture (Phase 1)
- Unified ARN-centric store architecture with comprehensive documentation
- Add ARN fields to STS and Tenant models for unified store
- Update IAM builders to use WamiArnBuilder for opaque ARNs
- Major refactor to pure function architecture
- Complete service layer implementation with 23 services
- Add 21 working examples with comprehensive documentation
- Implement Identity Provider Module (Issue #19)

### 🐛 Bug Fixes

- Replace rustyiam with wami in doc examples
- Update tests to work with new client API
- Add clippy allow for result_large_err in ARN parsing
- Correct doctest imports and ARN reconstruction
- Fix rustdoc bare URL warning in identity provider model
- **ci:** Correct package name from rustyiam to wami in auto-release workflow

### 📚 Documentation

- Add multicloud implementation status tracker
- Update multicloud status - implementation complete ✅
- Fix broken intra-doc links in IAM, STS, and SSO Admin modules
- Add comprehensive documentation structure
- Phase 2 ARN-Centric Architecture COMPLÉTÉE 🎉
- Fix all doc test examples for new architecture

### 🚜 Refactor

- Rename project from rustyiam to WAMI (Who Am I)
- Reorganize store module structure
- Extract resource builders from client logic
- **user:** Migrate User to self-contained resource structure
- Move user resource from resources/ to iam/
- Remove redundant builders/ directory
- Migrate group resource to self-contained structure
- Migrate role resource to self-contained structure
- Migrate policy resource to self-contained structure
- Migrate access_key resource to self-contained structure
- Migrate mfa_device resource to self-contained structure
- Migrate login_profile resource to self-contained structure
- **iam:** Convert all modules to self-contained structure
- **sts:** Convert to self-contained module structure
- Move tenant store to centralized store module
- Consolidate tenant authorization logic and simplify store traits

### ⚙️ Miscellaneous Tasks

- **release:** V0.9.0 [skip ci]

## [0.8.0] - 2025-10-26

### 🚀 Features

- Implement signing certificates module

### ⚙️ Miscellaneous Tasks

- **release:** V0.8.0 [skip ci]

## [0.7.0] - 2025-10-26

### 🚀 Features

- Implement service-linked roles module

### ⚙️ Miscellaneous Tasks

- **release:** V0.7.0 [skip ci]

## [0.6.0] - 2025-10-26

### 🚀 Features

- Implement service-specific credentials module

### ⚙️ Miscellaneous Tasks

- **release:** V0.6.0 [skip ci]

## [0.5.0] - 2025-10-26

### 🚀 Features

- Implement server certificates module for SSL/TLS certificate management

### ⚙️ Miscellaneous Tasks

- **release:** V0.5.0 [skip ci]

## [0.4.0] - 2025-10-26

### 🚀 Features

- Implement IAM reports module (credential reports and account summary)

### ⚙️ Miscellaneous Tasks

- **release:** V0.4.0 [skip ci]

## [0.3.0] - 2025-10-25

### 🚀 Features

- Implement IAM policy evaluation/simulation module

### ⚙️ Miscellaneous Tasks

- **release:** V0.3.0 [skip ci]

## [0.2.7] - 2025-10-25

### 🚜 Refactor

- Implement IAM resource tagging using IamStore trait

### ⚙️ Miscellaneous Tasks

- **release:** V0.2.7 [skip ci]

## [0.2.6] - 2025-10-25

### 🚜 Refactor

- Implement IAM policy methods using IamStore trait

### ⚙️ Miscellaneous Tasks

- **release:** V0.2.6 [skip ci]

## [0.2.5] - 2025-10-25

### 🚜 Refactor

- Implement login profile (password) methods using IamStore trait

### ⚙️ Miscellaneous Tasks

- **release:** V0.2.5 [skip ci]

## [0.2.4] - 2025-10-25

### 🚜 Refactor

- Implement roles methods using IamStore trait

### ⚙️ Miscellaneous Tasks

- **release:** V0.2.4 [skip ci]

## [0.2.3] - 2025-10-25

### 🚜 Refactor

- Implement MFA devices methods using IamStore trait

### ⚙️ Miscellaneous Tasks

- **release:** V0.2.3 [skip ci]

## [0.2.2] - 2025-10-25

### 🚜 Refactor

- Implement groups methods using IamStore trait

### ⚙️ Miscellaneous Tasks

- **release:** V0.2.2 [skip ci]

## [0.2.1] - 2025-10-25

### 🚜 Refactor

- Implement access_keys methods using IamStore trait

### ⚙️ Miscellaneous Tasks

- **release:** V0.2.1 [skip ci]

## [0.2.0] - 2025-10-25

### 🚀 Features

- Initial AWS IAM, STS, and SSO Admin operations library
- Complete in-memory AWS IAM, STS, and SSO Admin implementation
- Implement trait-based store architecture for easy backend swapping
- Complete trait-based architecture refactoring for STS and SSO Admin
- Implement dynamic AWS account ID generation
- Add account ID retrieval and logging capabilities
- Add AWS environment variable logging and export functionality
- Rename package to rustyiam and set MSRV to 1.81.0
- Add automatic version bumping and docs deployment workflow
- **hooks:** Add conventional commit template hook

### 🐛 Bug Fixes

- Correct AWS SSO Admin SDK package name
- Resolve build errors and ensure CI compatibility
- Resolve all clippy warnings for CI
- Remove Cargo.lock for library crate
- **ci:** Resolve workflow issues and add setup guide
- **ci:** Correct all GitHub Actions workflow issues
- **ci:** Make codecov upload optional and non-blocking
- **ci:** Simplify release workflow
- Replace git-cliff-action with manual install due to Debian Buster EOL

### 📚 Documentation

- Add comprehensive rustdoc with examples and reorganize README
- Add versioning section to README
- Add comprehensive versioning and release documentation
- **hooks:** Update README with prepare-commit-msg hook info
- Add repository setup reference to README

### 🚜 Refactor

- **ci:** Simplify workfkow

### 🧪 Testing

- Add comprehensive unit and integration tests

### ⚙️ Miscellaneous Tasks

- Modernize and enhance GitHub Actions workflows
- Add pre-commit hooks for code quality
- Update MSRV to 1.86.0
- Update MSRV to 1.90.0
- **release:** V0.2.0 [skip ci]

<!-- generated by git-cliff -->
