//! Server Certificate Resource Module
//!
//! This module provides self-contained handling of IAM server certificate resources.

pub mod builder;
pub mod model;
// pub mod operations; // TODO: Fix model ref
pub mod requests;

pub use model::*;
pub use requests::*;
