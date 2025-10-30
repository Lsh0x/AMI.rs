//! IAM Policy management

pub mod attachment;
pub mod inline;
pub mod policy;

// Re-export types for convenience
pub use attachment::*;
pub use inline::*;
pub use policy::{Policy, PolicyBuilder};

