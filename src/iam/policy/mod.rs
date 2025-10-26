//! Policy Resource Module
//!
//! This module provides self-contained handling of IAM policy resources.

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

pub use model::Policy;
pub use requests::{
    CreatePolicyRequest, ListPoliciesRequest, ListPoliciesResponse, UpdatePolicyRequest,
};
