//! Hybrid Cloud Setup
//!
//! This example demonstrates:
//! - Implementing a custom CloudProvider for on-premise systems
//! - Mixing custom providers with public cloud providers
//! - Federating identities across hybrid environments
//!
//! Scenario: Company with on-premise datacenter and AWS cloud.
//!
//! Run with: `cargo run --example 11_hybrid_cloud_setup`

use std::sync::{Arc, RwLock};
use wami::error::Result;
use wami::provider::{AwsProvider, CloudProvider, ResourceLimits, ResourceType};
use wami::service::UserService;
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::{CreateUserRequest, ListUsersRequest};

// === CUSTOM ON-PREMISE PROVIDER ===
#[derive(Debug, Clone)]
struct OnPremiseProvider {
    #[allow(dead_code)]
    datacenter_id: String,
    limits: ResourceLimits,
}

impl OnPremiseProvider {
    fn new(datacenter_id: String) -> Self {
        Self {
            datacenter_id,
            limits: ResourceLimits::default(),
        }
    }
}

impl CloudProvider for OnPremiseProvider {
    fn name(&self) -> &str {
        "onpremise"
    }

    fn generate_resource_identifier(
        &self,
        resource_type: ResourceType,
        account_id: &str,
        path: &str,
        name: &str,
    ) -> String {
        let resource_name = match resource_type {
            ResourceType::User => "user",
            ResourceType::Group => "group",
            ResourceType::Role => "role",
            ResourceType::Policy => "policy",
            _ => "resource",
        };
        format!(
            "arn:onprem:iam::{}:{}{}{}",
            account_id, resource_name, path, name
        )
    }

    fn generate_resource_id(&self, resource_type: ResourceType) -> String {
        let prefix = match resource_type {
            ResourceType::User => "ONPU",
            ResourceType::Group => "ONPG",
            ResourceType::Role => "ONPR",
            ResourceType::Policy => "ONPP",
            _ => "ONPR",
        };
        let uuid_part = uuid::Uuid::new_v4()
            .to_string()
            .replace('-', "")
            .chars()
            .take(17)
            .collect::<String>()
            .to_uppercase();
        format!("{}{}", prefix, uuid_part)
    }

    fn resource_limits(&self) -> &ResourceLimits {
        &self.limits
    }

    fn validate_service_name(&self, _service: &str) -> Result<()> {
        // On-premise systems can have custom service names
        Ok(())
    }

    fn validate_path(&self, path: &str) -> Result<()> {
        // Use AWS-style path validation for consistency
        if !path.starts_with('/') || !path.ends_with('/') {
            return Err(wami::error::AmiError::InvalidParameter {
                message: format!(
                    "Invalid path: '{}'. Paths must start and end with '/'",
                    path
                ),
            });
        }
        Ok(())
    }

    fn generate_service_linked_role_name(
        &self,
        service_name: &str,
        custom_suffix: Option<&str>,
    ) -> String {
        if let Some(suffix) = custom_suffix {
            format!("OnPremServiceRoleFor{}_{}", service_name, suffix)
        } else {
            format!("OnPremServiceRoleFor{}", service_name)
        }
    }

    fn generate_service_linked_role_path(&self, service_name: &str) -> String {
        format!("/onprem-service-role/{}/", service_name)
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Hybrid Cloud Setup ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // === INITIALIZE PROVIDERS (for ARN transformation) ===
    println!("Step 1: Note about providers...\n");

    let _onprem_provider = Arc::new(OnPremiseProvider::new("dc1-prod".to_string()));
    let _aws_provider = Arc::new(AwsProvider::new());

    println!("✓ Providers can be used for ARN transformation:");
    println!("  - On-Premise (datacenter: dc1-prod)");
    println!("  - AWS (public cloud)");
    println!("  - Services now use WamiContext instead of providers");

    // Create on-premise context
    let onprem_context = wami::context::WamiContext::builder()
        .instance_id("dc1-prod")
        .tenant_path(wami::arn::TenantPath::single("onprem"))
        .caller_arn(
            wami::arn::WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(wami::arn::TenantPath::single("onprem"))
                .wami_instance("dc1-prod")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Create AWS context
    let aws_context = wami::context::WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(wami::arn::TenantPath::single("aws"))
        .caller_arn(
            wami::arn::WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(wami::arn::TenantPath::single("aws"))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // === CREATE USER IN ON-PREMISE ===
    println!("\n\nStep 2: Creating user in on-premise environment...\n");

    let user_service = UserService::new(store.clone());

    let onprem_req = CreateUserRequest {
        user_name: "alice-onprem".to_string(),
        path: Some("/employees/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            wami::types::Tag {
                key: "Environment".to_string(),
                value: "OnPremise".to_string(),
            },
            wami::types::Tag {
                key: "Datacenter".to_string(),
                value: "dc1-prod".to_string(),
            },
        ]),
    };

    let alice_onprem = user_service
        .create_user(&onprem_context, onprem_req)
        .await?;
    println!("✓ Created alice-onprem in on-premise:");
    println!("  - ARN: {}", alice_onprem.arn);
    println!("  - WAMI ARN: {}", alice_onprem.wami_arn);
    println!("  - Resource ID: {}", alice_onprem.user_id);

    // === CREATE USER IN AWS ===
    println!("\n\nStep 3: Creating user in AWS cloud...\n");

    let aws_req = CreateUserRequest {
        user_name: "alice-cloud".to_string(),
        path: Some("/employees/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            wami::types::Tag {
                key: "Environment".to_string(),
                value: "AWS".to_string(),
            },
            wami::types::Tag {
                key: "Region".to_string(),
                value: "us-east-1".to_string(),
            },
        ]),
    };

    let alice_aws = user_service.create_user(&aws_context, aws_req).await?;
    println!("✓ Created alice-cloud in AWS:");
    println!("  - ARN: {}", alice_aws.arn);
    println!("  - WAMI ARN: {}", alice_aws.wami_arn);
    println!("  - Resource ID: {}", alice_aws.user_id);

    // === DEMONSTRATE FEDERATED IDENTITY ===
    println!("\n\nStep 4: Understanding hybrid identity federation...\n");

    println!("Both identities represent the same person (alice@company.com):");
    println!();
    println!("On-Premise Identity:");
    println!("  - ARN: {}", alice_onprem.arn);
    println!("  - For: Legacy applications, internal systems");
    println!();
    println!("Cloud Identity:");
    println!("  - ARN: {}", alice_aws.arn);
    println!("  - For: Cloud-native applications, external APIs");

    // === LIST ALL USERS ===
    println!("\n\nStep 4: Unified view across hybrid environment...\n");

    let (all_users, _, _) = user_service
        .list_users(ListUsersRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!(
        "✓ Total users across hybrid environment: {}",
        all_users.len()
    );
    for user in &all_users {
        let env = if user.wami_arn.to_string().contains("onprem") {
            "On-Premise"
        } else {
            "Cloud"
        };
        println!("  - {} ({}) → {}", user.user_name, env, user.wami_arn);
    }

    // === USE CASES ===
    println!("\n\nStep 5: Hybrid cloud use cases...\n");

    println!("WAMI context-based architecture enables:");
    println!("- On-premise to cloud migration paths");
    println!("- Unified identity management with WamiContext");
    println!("- Hybrid application architectures");
    println!("- Edge computing with centralized IAM");
    println!("- Consistent resource identification with WAMI ARNs");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- WamiContext replaces provider-specific service configuration");
    println!("- Use different contexts for different environments");
    println!("- WAMI ARNs provide unified identity layer");
    println!("- Same store works across all contexts");
    println!("- Providers can still be used for ARN transformation when needed");

    Ok(())
}
