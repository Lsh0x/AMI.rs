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
//! use wami::store::memory::InMemoryWamiStore;
//! use wami::store::traits::UserStore;
//! use wami::provider::AwsProvider;
//! use wami::wami::identity::user::builder::build_user;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut store = InMemoryWamiStore::default();
//! let provider = AwsProvider::new();
//!
//! let user = build_user(
//!     "alice".to_string(),
//!     Some("/engineering/".to_string()),
//!     &provider,
//!     "123456789012",
//! );
//!
//! let created_user = store.create_user(user).await?;
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
