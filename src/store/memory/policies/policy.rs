//! Policy Store Implementation for InMemoryWamiStore

use crate::error::Result;
use crate::store::memory::InMemoryWamiStore;
use crate::store::traits::PolicyStore;
use crate::types::PaginationParams;
use crate::wami::policies::Policy;
use async_trait::async_trait;

#[async_trait]
impl PolicyStore for InMemoryWamiStore {
    async fn create_policy(&mut self, policy: Policy) -> Result<Policy> {
        self.policies.insert(policy.arn.clone(), policy.clone());
        Ok(policy)
    }

    async fn get_policy(&self, policy_arn: &str) -> Result<Option<Policy>> {
        Ok(self.policies.get(policy_arn).cloned())
    }

    async fn update_policy(&mut self, policy: Policy) -> Result<Policy> {
        self.policies.insert(policy.arn.clone(), policy.clone());
        Ok(policy)
    }

    async fn delete_policy(&mut self, policy_arn: &str) -> Result<()> {
        self.policies.remove(policy_arn);
        Ok(())
    }

    async fn list_policies(
        &self,
        path_prefix: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) -> Result<(Vec<Policy>, bool, Option<String>)> {
        let mut policies: Vec<Policy> = self
            .policies
            .values()
            .filter(|policy| {
                if let Some(prefix) = path_prefix {
                    policy.path.starts_with(prefix)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Sort for consistent results
        policies.sort_by(|a, b| a.arn.cmp(&b.arn));

        if let Some(params) = pagination {
            let mut start_index = 0;
            if let Some(ref marker) = params.marker {
                if let Some(pos) = policies.iter().position(|p| &p.arn == marker) {
                    start_index = pos + 1;
                }
            }

            let max_items = params.max_items.unwrap_or(100).min(1000) as usize;
            let end_index = (start_index + max_items).min(policies.len());
            let is_truncated = end_index < policies.len();
            let next_marker = if is_truncated {
                policies.get(end_index - 1).map(|p| p.arn.clone())
            } else {
                None
            };

            policies = policies[start_index..end_index].to_vec();
            Ok((policies, is_truncated, next_marker))
        } else {
            Ok((policies, false, None))
        }
    }
}
