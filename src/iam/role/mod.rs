//! Role Resource Module
//!
//! This module provides self-contained handling of IAM role resources.

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

pub use model::Role;
pub use requests::{CreateRoleRequest, ListRolesRequest, ListRolesResponse, UpdateRoleRequest};
