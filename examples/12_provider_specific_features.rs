//! Provider-Specific Features
//!
//! This example demonstrates:
//! - Leveraging unique features of each cloud provider
//! - AWS IAM roles with trust policies
//! - GCP service accounts
//! - Azure managed identities
//!
//! Scenario: Using provider-specific identity patterns.
//!
//! Run with: `cargo run --example 12_provider_specific_features`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::RoleService;
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::role::requests::{CreateRoleRequest, ListRolesRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Provider-Specific Features ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create AWS context
    let aws_context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single(10000001)) // Numeric tenant ID for AWS
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(10000001))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Create GCP context
    let gcp_context = WamiContext::builder()
        .instance_id("my-gcp-project")
        .tenant_path(TenantPath::single(20000001)) // Numeric tenant ID for GCP
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(20000001))
                .wami_instance("my-gcp-project")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Create Azure context
    let azure_context = WamiContext::builder()
        .instance_id("my-subscription")
        .tenant_path(TenantPath::single(30000001)) // Numeric tenant ID for Azure
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(30000001))
                .wami_instance("my-subscription")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    let role_service = RoleService::new(store.clone());

    // === AWS: IAM ROLES WITH TRUST POLICIES ===
    println!("Step 1: AWS IAM Role with Trust Policy...\n");

    let aws_trust_policy = r#"{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": {"Service": "lambda.amazonaws.com"},
    "Action": "sts:AssumeRole"
  }]
}"#;

    let aws_role_req = CreateRoleRequest {
        role_name: "lambda-execution-role".to_string(),
        path: Some("/service-roles/".to_string()),
        assume_role_policy_document: aws_trust_policy.to_string(),
        description: Some("Role for Lambda function execution".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: None,
    };

    let aws_role = role_service.create_role(&aws_context, aws_role_req).await?;
    println!("✓ Created AWS Lambda execution role:");
    println!("  - ARN: {}", aws_role.arn);
    println!("  - WAMI ARN: {}", aws_role.wami_arn);
    println!("  - Trust Policy: Allows lambda.amazonaws.com to assume");
    println!("  - Use case: Serverless function execution");

    // === GCP: SERVICE ACCOUNT PATTERN ===
    println!("\n\nStep 2: GCP Service Account Pattern...\n");

    let gcp_trust_policy = r#"{
  "bindings": [{
    "role": "roles/iam.serviceAccountUser",
    "members": ["serviceAccount:compute@my-gcp-project.iam.gserviceaccount.com"]
  }]
}"#;

    let gcp_role_req = CreateRoleRequest {
        role_name: "compute-service-account".to_string(),
        path: Some("/service-accounts/".to_string()),
        assume_role_policy_document: gcp_trust_policy.to_string(),
        description: Some("Service account for Compute Engine".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: None,
    };

    let gcp_role = role_service.create_role(&gcp_context, gcp_role_req).await?;
    println!("✓ Created GCP service account:");
    println!("  - ARN: {}", gcp_role.arn);
    println!("  - WAMI ARN: {}", gcp_role.wami_arn);
    println!("  - Bindings: Compute Engine service usage");
    println!("  - Use case: VM instance identity");

    // === AZURE: MANAGED IDENTITY PATTERN ===
    println!("\n\nStep 3: Azure Managed Identity Pattern...\n");

    let azure_trust_policy = r#"{
  "properties": {
    "principalType": "ServicePrincipal",
    "tenantId": "tenant-id",
    "type": "SystemAssigned"
  }
}"#;

    let azure_role_req = CreateRoleRequest {
        role_name: "app-managed-identity".to_string(),
        path: Some("/managed-identities/".to_string()),
        assume_role_policy_document: azure_trust_policy.to_string(),
        description: Some("Managed identity for App Service".to_string()),
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: None,
    };

    let azure_role = role_service
        .create_role(&azure_context, azure_role_req)
        .await?;
    println!("✓ Created Azure managed identity:");
    println!("  - ARN: {}", azure_role.arn);
    println!("  - Type: System-assigned managed identity");
    println!("  - Use case: App Service authentication");

    // === COMPARE PATTERNS ===
    println!("\n\nStep 4: Comparing provider identity patterns...\n");

    println!("AWS IAM Roles:");
    println!("- Trust policies define who can assume the role");
    println!("- Used for cross-account and service-to-service access");
    println!("- Temporary credentials via STS");
    println!();
    println!("GCP Service Accounts:");
    println!("- Special type of account for applications");
    println!("- IAM bindings control permissions");
    println!("- Can be impersonated by other principals");
    println!();
    println!("Azure Managed Identities:");
    println!("- Automatically managed by Azure");
    println!("- System-assigned (bound to resource) or User-assigned");
    println!("- No credential management required");

    // === DEMONSTRATE UNIFIED VIEW ===
    println!("\n\nStep 5: Unified view across providers...\n");

    let (all_roles, _, _) = role_service
        .list_roles(ListRolesRequest {
            path_prefix: None,
            pagination: None,
        })
        .await?;
    println!(
        "✓ Total roles/identities across all providers: {}",
        all_roles.len()
    );
    for role in &all_roles {
        let provider = if role.wami_arn.to_string().contains("aws") {
            "AWS"
        } else if role.wami_arn.to_string().contains("gcp") {
            "GCP"
        } else {
            "Azure"
        };
        println!("  - {} ({}) → {}", role.role_name, provider, role.wami_arn);
    }

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Each provider has unique identity patterns");
    println!("- WAMI abstracts differences while preserving provider semantics");
    println!("- Trust policies define security boundaries");
    println!("- Choose patterns based on provider strengths");

    Ok(())
}
