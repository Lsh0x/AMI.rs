//! Credentials Service Module
//!
//! Services for managing IAM credentials (access keys, MFA devices, login profiles, certificates).

pub mod access_key;
pub mod login_profile;
pub mod mfa_device;
pub mod server_certificate;
pub mod service_credential;
pub mod signing_certificate;

pub use access_key::AccessKeyService;
pub use login_profile::LoginProfileService;
pub use mfa_device::MfaDeviceService;
pub use server_certificate::ServerCertificateService;
pub use service_credential::ServiceCredentialService;
pub use signing_certificate::SigningCertificateService;
