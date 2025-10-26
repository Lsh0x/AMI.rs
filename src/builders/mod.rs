//! Resource Builders
//!
//! This module contains pure functions for building IAM, STS, and SSO Admin resources.
//! These builders separate the business logic of constructing resources from the storage logic.
//!
//! ## Design Principles
//!
//! - **Pure Functions**: All builders are pure functions that don't have side effects
//! - **Separation of Concerns**: Business logic (ARN generation, defaults) is separated from storage
//! - **Testability**: Easy to test without mocking storage
//! - **Composability**: Builders can be composed and chained
//!
//! ## Example
//!
//! ```rust
//! use wami::builders::user::build_user;
//! use wami::provider::AwsProvider;
//! use std::sync::Arc;
//!
//! let provider = Arc::new(AwsProvider::new());
//! let user = build_user(
//!     "alice".to_string(),
//!     Some("/engineering/".to_string()),
//!     None,
//!     None,
//!     provider.as_ref(),
//!     "123456789012",
//! );
//!
//! assert_eq!(user.user_name, "alice");
//! assert!(user.arn.contains("arn:aws:iam::123456789012:user/engineering/alice"));
//! ```

pub mod user;

// Re-export commonly used builder functions
pub use user::{add_provider_to_user, build_user, update_user};
