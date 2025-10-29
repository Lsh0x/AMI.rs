//! Policy Store Trait
//!
//! Focused trait for policy-related storage operations

use crate::error::Result;
use crate::types::PaginationParams;
use crate::wami::policies::Policy;
use async_trait::async_trait;

/// Store trait for IAM policy operations
#[async_trait]
pub trait PolicyStore: Send + Sync {
    /// Create a new policy
    async fn create_policy(&mut self, policy: Policy) -> Result<Policy>;

    /// Get a policy by ARN
    async fn get_policy(&self, policy_arn: &str) -> Result<Option<Policy>>;

    /// Update an existing policy
    async fn update_policy(&mut self, policy: Policy) -> Result<Policy>;

    /// Delete a policy
    async fn delete_policy(&mut self, policy_arn: &str) -> Result<()>;

    /// List policies with optional filtering and pagination
    async fn list_policies(
        &self,
        scope: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Policy>, bool, Option<String>)>;
}
