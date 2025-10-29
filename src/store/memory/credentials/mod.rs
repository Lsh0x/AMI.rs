//! Credentials Sub-Trait Implementations
//!
//! Implements all credential-related stores for InMemoryWamiStore.

pub mod access_key;
pub mod login_profile;
pub mod mfa_device;
// TODO: Temporarily disabled during refactor - field mismatches to fix
// pub mod server_certificate;
// pub mod signing_certificate;
// pub mod service_credential;

#[cfg(test)]
mod tests;
