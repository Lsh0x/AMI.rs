//! Identity Sub-Trait Implementations
//!
//! Implements all identity-related stores for InMemoryWamiStore.

pub mod group;
pub mod role;
pub mod service_linked_role;
pub mod user;

#[cfg(test)]
mod tests;
