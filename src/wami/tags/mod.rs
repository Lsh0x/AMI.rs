//! Resource tagging operations

pub mod operations;
pub mod requests;

// Re-export request types
pub use requests::{ListResourceTagsRequest, TagResourceRequest, UntagResourceRequest};
