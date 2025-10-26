//! Store Trait Definitions
//!
//! This module contains the trait definitions for all store operations.
//! These traits define the interface that storage backends must implement.

mod iam;
mod sso_admin;
mod sts;

pub use iam::IamStore;
pub use sso_admin::SsoAdminStore;
pub use sts::StsStore;
