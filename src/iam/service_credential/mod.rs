//! Service Credential Resource Module
//!
//! This module provides self-contained handling of IAM service-specific credential resources.

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

pub use model::*;
pub use requests::*;
