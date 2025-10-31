//! Authentication and Authorization Services
//!
//! This module provides services for:
//! - **Authentication**: Validating credentials and creating contexts
//! - **Authorization**: Checking permissions based on IAM policies
//!
//! # Authentication Flow
//!
//! 1. User provides access key credentials
//! 2. `AuthenticationService` validates the credentials
//! 3. Loads the user and extracts instance/tenant from ARN
//! 4. Creates a `WamiContext` for subsequent operations
//!
//! # Authorization Flow
//!
//! 1. An authenticated `WamiContext` is provided
//! 2. `AuthorizationService` checks if action is allowed
//! 3. Root users bypass all checks (full access)
//! 4. Regular users are subject to policy evaluation
//! 5. Policies from user, groups, and roles are evaluated
//! 6. Deny overrides Allow
//!
//! # Example
//!
//! ```rust,no_run
//! use wami::{AuthenticationService, AuthorizationService, store::memory::InMemoryWamiStore, WamiArn};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
//!    
//!     // Authenticate
//!     let auth_service = AuthenticationService::new(store.clone());
//!     let context = auth_service
//!         .authenticate("access_key_id", "secret_access_key")
//!         .await?;
//!
//!     // Authorize
//!     let authz_service = AuthorizationService::new(store.clone());
//!     let resource: WamiArn = "arn:wami:iam:t1:wami:999:user/alice".parse()?;
//!     
//!     if authz_service.authorize(&context, "iam:GetUser", &resource).await? {
//!         println!("Access granted");
//!     } else {
//!         println!("Access denied");
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod authentication;
pub mod authorization;

pub use authentication::{hash_secret, verify_secret, AuthenticationService};
pub use authorization::AuthorizationService;
