//! Identity Store Traits
//!
//! Sub-traits for identity resource storage

mod group;
mod identity_provider;
mod role;
mod service_linked_role;
mod user;

pub use group::GroupStore;
pub use identity_provider::IdentityProviderStore;
pub use role::RoleStore;
pub use service_linked_role::ServiceLinkedRoleStore;
pub use user::UserStore;
