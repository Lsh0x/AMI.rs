//! IAM Report Resource Module
//!
//! This module provides self-contained handling of IAM reports including
//! credential reports and account summaries.

pub mod model;
pub mod operations;
pub mod requests;

pub use model::*;
pub use requests::*;
