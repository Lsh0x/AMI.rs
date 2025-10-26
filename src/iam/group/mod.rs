//! Group Resource Module
//!
//! This module provides self-contained handling of IAM group resources.

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

pub use model::Group;
pub use requests::{CreateGroupRequest, ListGroupsRequest, ListGroupsResponse, UpdateGroupRequest};
