//! AccessKey Resource Module
//!
//! This module provides self-contained handling of IAM access key resources.

pub mod builder;
pub mod model;
// pub mod operations; // TODO: Fix field mismatches in tests
pub mod requests;

pub use model::{AccessKey, AccessKeyLastUsed};
// Operations moved to service layer
// pub use operations::AccessKeyOperations;
pub use requests::{
    CreateAccessKeyRequest, ListAccessKeysRequest, ListAccessKeysResponse, UpdateAccessKeyRequest,
};
