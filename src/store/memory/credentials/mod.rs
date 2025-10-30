//! Credentials Sub-Trait Implementations
//!
//! Implements all credential-related stores for InMemoryWamiStore.

pub mod access_key;
pub mod login_profile;
pub mod mfa_device;
pub mod server_certificate;
pub mod service_credential;
pub mod signing_certificate;

#[cfg(test)]
mod tests;
