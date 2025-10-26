//! AccessKey Resource Module
//!
//! This module provides self-contained handling of IAM access key resources.

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

pub use model::{AccessKey, AccessKeyLastUsed};
pub use requests::{
    CreateAccessKeyRequest, ListAccessKeysRequest, ListAccessKeysResponse, UpdateAccessKeyRequest,
};
