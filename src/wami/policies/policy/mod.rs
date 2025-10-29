//! Policy Resource Module
//!
//! This module provides self-contained handling of IAM policy resources.

pub mod builder;
pub mod model;
// pub mod operations; // TODO: Fix field mismatches in tests
pub mod requests;

pub use model::Policy;
// Operations moved to pure functions
// pub use operations::PolicyOperations;
pub use requests::{
    CreatePolicyRequest, ListPoliciesRequest, ListPoliciesResponse, UpdatePolicyRequest,
};
