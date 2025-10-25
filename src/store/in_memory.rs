use crate::error::Result;
use crate::store::memory::InMemoryIamStore;
use crate::store::memory_sts_sso::{InMemorySsoAdminStore, InMemoryStsStore};
use crate::store::Store;
use async_trait::async_trait;

/// Main store implementation that combines all sub-stores
#[derive(Debug, Clone)]
pub struct InMemoryStore {
    pub account_id: String,
    pub iam_store: InMemoryIamStore,
    pub sts_store: InMemoryStsStore,
    pub sso_admin_store: InMemorySsoAdminStore,
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStore {
    pub fn new() -> Self {
        let account_id = crate::types::AwsConfig::generate_account_id();
        log::info!("Generated AWS account ID: {}", account_id);
        Self::log_aws_environment_variables(&account_id);
        Self {
            account_id: account_id.clone(),
            iam_store: InMemoryIamStore::with_account_id(account_id.clone()),
            sts_store: InMemoryStsStore::with_account_id(account_id.clone()),
            sso_admin_store: InMemorySsoAdminStore::default(),
        }
    }

    pub fn with_account_id(account_id: String) -> Self {
        log::info!("Using provided AWS account ID: {}", account_id);
        Self::log_aws_environment_variables(&account_id);
        Self {
            account_id: account_id.clone(),
            iam_store: InMemoryIamStore::with_account_id(account_id.clone()),
            sts_store: InMemoryStsStore::with_account_id(account_id.clone()),
            sso_admin_store: InMemorySsoAdminStore::default(),
        }
    }

    /// Log AWS environment variables for export
    fn log_aws_environment_variables(account_id: &str) {
        log::info!("AWS Environment Variables for export:");
        log::info!("  export AWS_ACCOUNT_ID={}", account_id);
        log::info!("  export AWS_DEFAULT_REGION=us-east-1");
        log::info!("  export AWS_REGION=us-east-1");
        log::info!("  export AWS_PROFILE=default");
        log::info!("");
        log::info!("To use with AWS CLI or other tools, run:");
        log::info!("  export AWS_ACCOUNT_ID={}", account_id);
        log::info!("  export AWS_DEFAULT_REGION=us-east-1");
        log::info!("");
    }

    /// Get the current AWS account ID
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    /// Get the current AWS account ID as a String
    pub fn account_id_string(&self) -> String {
        self.account_id.clone()
    }

    /// Get AWS environment variables as a HashMap for easy export
    pub fn aws_environment_variables(&self) -> std::collections::HashMap<String, String> {
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("AWS_ACCOUNT_ID".to_string(), self.account_id.clone());
        env_vars.insert("AWS_DEFAULT_REGION".to_string(), "us-east-1".to_string());
        env_vars.insert("AWS_REGION".to_string(), "us-east-1".to_string());
        env_vars.insert("AWS_PROFILE".to_string(), "default".to_string());
        env_vars
    }

    /// Print AWS environment variables to stdout for easy copying
    pub fn print_aws_environment_variables(&self) {
        println!("AWS Environment Variables:");
        println!("  export AWS_ACCOUNT_ID={}", self.account_id);
        println!("  export AWS_DEFAULT_REGION=us-east-1");
        println!("  export AWS_REGION=us-east-1");
        println!("  export AWS_PROFILE=default");
        println!();
        println!("To use with AWS CLI or other tools, run:");
        println!("  export AWS_ACCOUNT_ID={}", self.account_id);
        println!("  export AWS_DEFAULT_REGION=us-east-1");
        println!();
    }
}

#[async_trait]
impl Store for InMemoryStore {
    type IamStore = InMemoryIamStore;
    type StsStore = InMemoryStsStore;
    type SsoAdminStore = InMemorySsoAdminStore;

    async fn iam_store(&mut self) -> Result<&mut Self::IamStore> {
        Ok(&mut self.iam_store)
    }

    async fn sts_store(&mut self) -> Result<&mut Self::StsStore> {
        Ok(&mut self.sts_store)
    }

    async fn sso_admin_store(&mut self) -> Result<&mut Self::SsoAdminStore> {
        Ok(&mut self.sso_admin_store)
    }
}
