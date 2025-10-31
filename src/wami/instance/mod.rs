//! WAMI Instance Management
//!
//! A WAMI instance represents an isolated environment similar to an AWS account.
//! Each instance has:
//! - A unique instance ID
//! - A root user with full administrative access
//! - Root user credentials (access key + secret key)

pub mod bootstrap;

pub use bootstrap::{InstanceBootstrap, RootCredentials};
