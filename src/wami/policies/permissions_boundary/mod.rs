//! Permissions boundary management

pub mod model;
pub mod operations;
pub mod requests;

// Re-export types
pub use model::PermissionsBoundary;
pub use requests::{
    DeletePermissionsBoundaryRequest, PrincipalType, PutPermissionsBoundaryRequest,
};
