//! Role Resource Module
//!
//! This module provides self-contained handling of IAM role resources.

pub mod builder;
pub mod model;
// pub mod operations; // TODO: Fix field mismatches in tests
pub mod requests;

pub use model::Role;
// Operations moved to service layer
// pub use operations::RoleOperations;
pub use requests::{CreateRoleRequest, ListRolesRequest, ListRolesResponse, UpdateRoleRequest};
