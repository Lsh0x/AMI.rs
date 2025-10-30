//! SSO Admin Services
//!
//! This module contains services for SSO Administration operations:
//! - `instance` - SSO Instance management
//! - `permission_set` - Permission Set management
//! - `account_assignment` - Account Assignment management
//! - `application` - Application management
//! - `trusted_token_issuer` - Trusted Token Issuer management

pub mod account_assignment;
pub mod application;
pub mod instance;
pub mod permission_set;
pub mod trusted_token_issuer;

pub use account_assignment::AccountAssignmentService;
pub use application::ApplicationService;
pub use instance::InstanceService;
pub use permission_set::PermissionSetService;
pub use trusted_token_issuer::TrustedTokenIssuerService;
