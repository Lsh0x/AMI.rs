//! LoginProfile Resource Module
//!
//! This module provides self-contained handling of IAM login profile resources.

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

pub use model::LoginProfile;
pub use requests::{CreateLoginProfileRequest, GetLoginProfileRequest, UpdateLoginProfileRequest};
