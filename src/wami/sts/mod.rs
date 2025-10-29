//! Security Token Service (STS) - temporary credentials and federation

pub mod assume_role;
pub mod credentials;
pub mod federation;
pub mod identity;
pub mod session;
pub mod session_token;

// #[cfg(test)]
// pub mod tests;  // Temporarily disabled - will rewrite with pure function tests

// Re-export main types
pub use assume_role::{AssumeRoleRequest, AssumeRoleResponse};
pub use credentials::Credentials;
pub use identity::model::CallerIdentity; // Model types
pub use session::StsSession;
pub use session_token::GetSessionTokenRequest;
// Note: Some types were in operations modules and may need to be re-exported from requests.rs
