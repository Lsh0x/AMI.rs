//! User Resource Module
//!
//! Complete user resource management including model, builder, operations, and requests.
//!
//! ## Structure
//!
//! - `model` - User struct and domain validation
//! - `builder` - Pure functions for constructing User instances
//! - `requests` - Request/Response DTOs
//! - `operations` - IamClient methods for user operations
//!
//! ## Example
//!
//! ```rust
//! use wami::iam::user::{User, CreateUserRequest};
//! use wami::{IamClient, InMemoryStore};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = InMemoryStore::new();
//! let mut client = IamClient::new(store);
//!
//! let request = CreateUserRequest {
//!     user_name: "alice".to_string(),
//!     path: Some("/engineering/".to_string()),
//!     permissions_boundary: None,
//!     tags: None,
//! };
//!
//! let response = client.create_user(request).await?;
//! # Ok(())
//! # }
//! ```

pub mod builder;
pub mod model;
// pub mod operations; // TODO: Fix field mismatches in tests
pub mod requests;

// Re-export main types
pub use model::User;
// Operations moved to service layer - pure functions remain here
// pub use operations::UserOperations;
pub use requests::{CreateUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest};
