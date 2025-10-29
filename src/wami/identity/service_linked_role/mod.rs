//! Service Linked Role Resource Module
//!
//! This module provides self-contained handling of IAM service-linked role resources.

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

pub use model::*;
pub use requests::*;
