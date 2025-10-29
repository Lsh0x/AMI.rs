//! Credentials Store Traits
//!
//! Sub-traits for credential resource storage

mod access_key;
mod login_profile;
mod mfa_device;
mod server_certificate;
mod service_credential;
mod signing_certificate;

pub use access_key::AccessKeyStore;
pub use login_profile::LoginProfileStore;
pub use mfa_device::MfaDeviceStore;
pub use server_certificate::ServerCertificateStore;
pub use service_credential::ServiceCredentialStore;
pub use signing_certificate::SigningCertificateStore;
