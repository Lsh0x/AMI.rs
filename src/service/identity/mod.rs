//! Identity Services
//!
//! Services for managing users, groups, roles, and service-linked roles.

pub mod group;
pub mod role;
pub mod service_linked_role;
pub mod user;

pub use group::GroupService;
pub use role::RoleService;
pub use service_linked_role::ServiceLinkedRoleService;
pub use user::UserService;
