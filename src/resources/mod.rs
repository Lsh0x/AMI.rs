//! Resource Modules
//!
//! Self-contained resource modules with model, builder, operations, and requests.
//!
//! Each resource is organized in its own directory with a consistent structure:
//! - `model.rs` - Domain model and validation
//! - `builder.rs` - Pure construction functions
//! - `requests.rs` - Request/Response DTOs
//! - `operations.rs` - Client operations
//! - `mod.rs` - Public API and re-exports

pub mod user;

// Re-export commonly used types
pub use user::User;
