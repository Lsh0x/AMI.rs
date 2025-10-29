//! Single Sign-On Administration
//!
//! Management of SSO permission sets, account assignments, and instances.

pub mod account_assignment;
pub mod application;
pub mod instance;
pub mod permission_set;
pub mod trusted_token_issuer;

// #[cfg(test)]
// pub mod tests;  // Temporarily disabled - will rewrite with pure function tests

// Re-export main types
pub use account_assignment::AccountAssignment;
pub use application::Application;
pub use instance::SsoInstance;
pub use permission_set::PermissionSet;
pub use trusted_token_issuer::TrustedTokenIssuer;
