//! Credential management: access keys, MFA devices, login profiles, and certificates

pub mod access_key;
pub mod mfa_device;
pub mod login_profile;
pub mod server_certificate;
pub mod signing_certificate;
pub mod service_credential;

// Re-export types for convenience
pub use access_key::{AccessKey, AccessKeyBuilder};
pub use mfa_device::{MfaDevice, MfaDeviceBuilder};
pub use login_profile::{LoginProfile, LoginProfileBuilder};
pub use server_certificate::{ServerCertificate, ServerCertificateBuilder, ServerCertificateMetadata};
pub use signing_certificate::SigningCertificate;
pub use service_credential::ServiceSpecificCredential;


