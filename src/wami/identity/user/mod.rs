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
//! use wami::wami::identity::user::builder::build_user;
//! use wami::{WamiContext, TenantPath, WamiArn, Service};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut store = InMemoryWamiStore::default();
//!
//! let context = WamiContext::builder()
//!     .instance_id("123456789012")
//!     .tenant_path(TenantPath::single(0))
//!     .caller_arn(
//!         WamiArn::builder()
//!             .service(Service::Iam)
//!             .tenant_path(TenantPath::single(0))
//!             .wami_instance("123456789012")
//!             .resource("user", "admin")
//!             .build()?,
//!     )
//!     .is_root(false)
//!     .build()?;
//!
//! let user = build_user(
//!     "alice".to_string(),
//!     Some("/engineering/".to_string()),
//!     &context,
//! )?;
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
