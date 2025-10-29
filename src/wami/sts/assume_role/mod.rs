//! Assume Role Module
//!
//! This module provides self-contained handling of IAM role assumption operations.

pub mod model;
// pub mod operations; // TODO: Fix field/ResourceType issues
pub mod requests;

pub use model::*;
pub use requests::*;
