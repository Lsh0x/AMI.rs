//! Identity Store Traits
//!
//! Sub-traits for identity resource storage

mod group;
mod role;
mod service_linked_role;
mod user;

pub use group::GroupStore;
pub use role::RoleStore;
pub use service_linked_role::ServiceLinkedRoleStore;
pub use user::UserStore;
