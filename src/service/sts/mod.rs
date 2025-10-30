//! STS Service Module
//!
//! Services for Security Token Service operations.

pub mod assume_role;
pub mod federation;
pub mod identity;
pub mod session;
pub mod session_token;

pub use assume_role::AssumeRoleService;
pub use federation::FederationService;
pub use identity::{GetCallerIdentityRequest, GetCallerIdentityResponse, IdentityService};
pub use session::SessionService;
pub use session_token::{GetSessionTokenResponse, SessionTokenService};
