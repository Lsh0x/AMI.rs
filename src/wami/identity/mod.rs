//! Identity management: users, groups, roles, and service-linked roles

pub mod user;
pub mod group;
pub mod role;
pub mod root_user;
pub mod service_linked_role;

// Re-export types for convenience
pub use user::{User, UserBuilder};
pub use group::{Group, GroupBuilder};
pub use role::{Role, RoleBuilder};
pub use root_user::RootUser;
pub use service_linked_role::DeletionTaskInfo;


