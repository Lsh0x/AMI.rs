//! Multi-tenant management and isolation

pub mod authorization;
pub mod hierarchy;
pub mod model;
pub mod operations; // Pure functions (was client.rs)

// #[cfg(test)]
// pub mod tests;  // Temporarily disabled - will rewrite with pure function tests

// Re-export main types
pub use authorization::{check_tenant_permission, TenantAction};
pub use model::{
    BillingInfo, QuotaMode, Tenant, TenantId, TenantQuotas, TenantStatus, TenantType, TenantUsage,
};
// TenantClient removed - use pure functions in operations module instead
