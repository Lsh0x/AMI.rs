//! Group Resource Module
//!
//! This module provides self-contained handling of IAM group resources.

pub mod builder;
pub mod model;
// pub mod operations; // TODO: Fix field mismatches in tests
pub mod requests;

pub use model::Group;
// Operations moved to service layer
// pub use operations::GroupOperations;
pub use requests::{CreateGroupRequest, ListGroupsRequest, ListGroupsResponse, UpdateGroupRequest};
