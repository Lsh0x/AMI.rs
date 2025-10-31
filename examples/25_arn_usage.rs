//! # ARN Usage Example
//!
//! This example demonstrates the WAMI ARN system, including:
//! - Building WAMI native and cloud-synced ARNs
//! - Parsing ARN strings
//! - Transforming to provider-specific formats
//! - Querying resources by ARN prefix
//! - Handling hierarchical tenants
//!
//! Run with: `cargo run --example 25_arn_usage`

use std::str::FromStr;
use wami::arn::{
    parse_arn, ArnTransformer, AwsArnTransformer, AzureArnTransformer, GcpArnTransformer,
    ScalewayArnTransformer, Service, TenantPath, WamiArn,
};
use wami::error::Result;

#[allow(clippy::result_large_err)]
fn main() -> Result<()> {
    println!("=== WAMI ARN Usage Examples ===\n");

    // 1. Building WAMI Native ARNs
    building_native_arns()?;

    // 2. Building Cloud-Synced ARNs
    building_cloud_synced_arns()?;

    // 3. Parsing ARNs
    parsing_arns()?;

    // 4. Transforming to Provider Formats
    transforming_to_provider_formats()?;

    // 5. Querying by ARN Prefix
    querying_by_prefix()?;

    // 6. Hierarchical Tenants
    hierarchical_tenants()?;

    // 7. ARN Introspection
    arn_introspection()?;

    println!("\n=== Example Complete ===");
    Ok(())
}

/// Demonstrates building WAMI native ARNs (no cloud sync).
#[allow(clippy::result_large_err)]
fn building_native_arns() -> Result<()> {
    println!("## 1. Building WAMI Native ARNs\n");

    // Simple single-tenant ARN
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant("t1")
        .wami_instance("999888777")
        .resource("user", "77557755")
        .build()?;

    println!("Single tenant ARN:");
    println!("  {}\n", arn);

    // Multi-tenant hierarchy ARN
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_hierarchy(vec!["t1", "t2", "t3"])
        .wami_instance("999888777")
        .resource("role", "12345678")
        .build()?;

    println!("Multi-tenant hierarchy ARN:");
    println!("  {}\n", arn);

    // Using service_str for custom services
    let arn = WamiArn::builder()
        .service_str("custom-service")
        .tenant("t1")
        .wami_instance("999888777")
        .resource("custom-resource", "res123")
        .build()?;

    println!("Custom service ARN:");
    println!("  {}\n", arn);

    // STS service
    let arn = WamiArn::builder()
        .service(Service::Sts)
        .tenant("t1")
        .wami_instance("999888777")
        .resource("session", "sess-abc123")
        .build()?;

    println!("STS session ARN:");
    println!("  {}\n", arn);

    Ok(())
}

/// Demonstrates building cloud-synced ARNs.
#[allow(clippy::result_large_err)]
fn building_cloud_synced_arns() -> Result<()> {
    println!("## 2. Building Cloud-Synced ARNs\n");

    // AWS-synced ARN
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_hierarchy(vec!["t1", "t2", "t3"])
        .wami_instance("999888777")
        .cloud_provider("aws", "223344556677")
        .resource("user", "77557755")
        .build()?;

    println!("AWS-synced ARN:");
    println!("  {}", arn);
    println!("  Cloud synced: {}", arn.is_cloud_synced());
    println!("  Provider: {}\n", arn.provider().unwrap());

    // GCP-synced ARN
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_hierarchy(vec!["t1", "t2", "t3"])
        .wami_instance("999888777")
        .cloud_provider("gcp", "554433221")
        .resource("serviceAccount", "77557755")
        .build()?;

    println!("GCP-synced ARN:");
    println!("  {}\n", arn);

    // Azure-synced ARN
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant("t1")
        .wami_instance("999888777")
        .cloud_provider("azure", "sub-12345-67890")
        .resource("user", "77557755")
        .build()?;

    println!("Azure-synced ARN:");
    println!("  {}\n", arn);

    // Scaleway-synced ARN
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant("t1")
        .wami_instance("999888777")
        .cloud_provider("scaleway", "112233445")
        .resource("user", "77557755")
        .build()?;

    println!("Scaleway-synced ARN:");
    println!("  {}\n", arn);

    Ok(())
}

/// Demonstrates parsing ARN strings.
#[allow(clippy::result_large_err)]
fn parsing_arns() -> Result<()> {
    println!("## 3. Parsing ARNs\n");

    // Parse WAMI native ARN
    let arn_str = "arn:wami:iam:t1/t2/t3:wami:999888777:user/77557755";
    let arn = WamiArn::from_str(arn_str)?;

    println!("Parsed WAMI native ARN:");
    println!("  Input:  {}", arn_str);
    println!("  Service: {}", arn.service);
    println!("  Tenant path: {}", arn.full_tenant_path());
    println!("  Instance ID: {}", arn.wami_instance_id);
    println!("  Resource type: {}", arn.resource_type());
    println!("  Resource ID: {}\n", arn.resource_id());

    // Parse cloud-synced ARN
    let arn_str = "arn:wami:iam:t1/t2/t3:wami:999888777:aws:223344556677:user/77557755";
    let arn = parse_arn(arn_str)?;

    println!("Parsed cloud-synced ARN:");
    println!("  Input:  {}", arn_str);
    println!("  Cloud synced: {}", arn.is_cloud_synced());
    println!("  Provider: {}", arn.provider().unwrap());
    println!("  Resource: {}\n", arn.resource);

    // Roundtrip test
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant("t1")
        .wami_instance("999888777")
        .resource("policy", "pol123")
        .build()?;

    let arn_str = arn.to_string();
    let parsed = WamiArn::from_str(&arn_str)?;

    println!("Roundtrip test:");
    println!("  Original: {}", arn_str);
    println!("  Parsed:   {}", parsed);
    println!("  Match:    {}\n", arn == parsed);

    Ok(())
}

/// Demonstrates transforming WAMI ARNs to provider-specific formats.
#[allow(clippy::result_large_err)]
fn transforming_to_provider_formats() -> Result<()> {
    println!("## 4. Transforming to Provider Formats\n");

    // AWS transformation
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_hierarchy(vec!["t1", "t2", "t3"])
        .wami_instance("999888777")
        .cloud_provider("aws", "223344556677")
        .resource("user", "77557755")
        .build()?;

    let transformer = AwsArnTransformer;
    let aws_arn = transformer.to_provider_arn(&arn)?;

    println!("AWS Transformation:");
    println!("  WAMI ARN: {}", arn);
    println!("  AWS ARN:  {}\n", aws_arn);

    // Parse AWS ARN back
    let info = transformer.from_provider_arn(&aws_arn)?;
    println!("  Parsed back:");
    println!("    Provider: {}", info.provider);
    println!("    Account ID: {}", info.account_id);
    println!("    Service: {}", info.service);
    println!(
        "    Resource: {}/{}\n",
        info.resource_type, info.resource_id
    );

    // GCP transformation
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant("t1")
        .wami_instance("999888777")
        .cloud_provider("gcp", "my-project-123")
        .resource("serviceAccount", "sa-77557755")
        .build()?;

    let transformer = GcpArnTransformer;
    let gcp_name = transformer.to_provider_arn(&arn)?;

    println!("GCP Transformation:");
    println!("  WAMI ARN:       {}", arn);
    println!("  GCP Resource:   {}\n", gcp_name);

    // Azure transformation
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant("t1")
        .wami_instance("999888777")
        .cloud_provider("azure", "sub-12345-67890")
        .resource("user", "77557755")
        .build()?;

    let transformer = AzureArnTransformer;
    let azure_id = transformer.to_provider_arn(&arn)?;

    println!("Azure Transformation:");
    println!("  WAMI ARN:     {}", arn);
    println!("  Azure ID:     {}\n", azure_id);

    // Scaleway transformation
    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant("t1")
        .wami_instance("999888777")
        .cloud_provider("scaleway", "org-112233")
        .resource("user", "77557755")
        .build()?;

    let transformer = ScalewayArnTransformer;
    let scw_resource = transformer.to_provider_arn(&arn)?;

    println!("Scaleway Transformation:");
    println!("  WAMI ARN:        {}", arn);
    println!("  Scaleway ID:     {}\n", scw_resource);

    Ok(())
}

/// Demonstrates querying resources by ARN prefix.
#[allow(clippy::result_large_err)]
fn querying_by_prefix() -> Result<()> {
    println!("## 5. Querying by ARN Prefix\n");

    let arns = vec![
        WamiArn::builder()
            .service(Service::Iam)
            .tenant_hierarchy(vec!["t1", "t2", "t3"])
            .wami_instance("999888777")
            .resource("user", "1001")
            .build()?,
        WamiArn::builder()
            .service(Service::Iam)
            .tenant_hierarchy(vec!["t1", "t2", "t3"])
            .wami_instance("999888777")
            .resource("role", "2001")
            .build()?,
        WamiArn::builder()
            .service(Service::Iam)
            .tenant_hierarchy(vec!["t1", "t2"])
            .wami_instance("999888777")
            .resource("user", "3001")
            .build()?,
        WamiArn::builder()
            .service(Service::Iam)
            .tenant_hierarchy(vec!["t1", "t2", "t3"])
            .wami_instance("999888777")
            .cloud_provider("aws", "223344556677")
            .resource("user", "4001")
            .build()?,
    ];

    println!("Sample ARNs:");
    for (i, arn) in arns.iter().enumerate() {
        println!("  [{}] {}", i + 1, arn);
    }
    println!();

    // Query by tenant prefix
    let prefix = "arn:wami:iam:t1/t2/t3:wami:999888777";
    println!("ARNs matching prefix '{}':", prefix);
    for arn in &arns {
        if arn.matches_prefix(prefix) {
            println!("  ✓ {}", arn);
        }
    }
    println!();

    // Query cloud-synced resources
    let prefix = "arn:wami:iam:t1/t2/t3:wami:999888777:aws";
    println!("AWS-synced ARNs:");
    for arn in &arns {
        if arn.matches_prefix(prefix) {
            println!("  ✓ {}", arn);
        }
    }
    println!();

    // Query by service and instance
    let prefix = "arn:wami:iam:t1/t2:wami:999888777";
    println!("ARNs in tenant t1/t2 (including descendants):");
    for arn in &arns {
        if arn.matches_prefix(prefix) {
            println!("  ✓ {}", arn);
        }
    }
    println!();

    Ok(())
}

/// Demonstrates hierarchical tenant operations.
#[allow(clippy::result_large_err)]
fn hierarchical_tenants() -> Result<()> {
    println!("## 6. Hierarchical Tenants\n");

    // Create tenant hierarchy
    let root = TenantPath::single("acme");
    let division = TenantPath::new(vec!["acme".to_string(), "engineering".to_string()]);
    let team = TenantPath::new(vec![
        "acme".to_string(),
        "engineering".to_string(),
        "backend".to_string(),
    ]);

    println!("Tenant Hierarchy:");
    println!("  Root:     {}", root);
    println!("  Division: {}", division);
    println!("  Team:     {}\n", team);

    // Check relationships
    println!("Relationships:");
    println!(
        "  Team is descendant of Division: {}",
        team.is_descendant_of(&division)
    );
    println!(
        "  Division is ancestor of Team: {}",
        division.is_ancestor_of(&team)
    );
    println!(
        "  Team is descendant of Root: {}",
        team.is_descendant_of(&root)
    );
    println!(
        "  Root is descendant of Team: {}",
        root.is_descendant_of(&team)
    );
    println!();

    // Create ARNs at different levels
    let root_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(root.clone())
        .wami_instance("999888777")
        .resource("policy", "company-policy")
        .build()?;

    let division_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(division.clone())
        .wami_instance("999888777")
        .resource("role", "eng-admin")
        .build()?;

    let team_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(team.clone())
        .wami_instance("999888777")
        .resource("user", "backend-dev")
        .build()?;

    println!("ARNs at different hierarchy levels:");
    println!("  Root ARN:     {}", root_arn);
    println!("  Division ARN: {}", division_arn);
    println!("  Team ARN:     {}\n", team_arn);

    // Check tenant membership
    println!("Tenant membership:");
    println!(
        "  Team ARN belongs to Division: {}",
        team_arn.belongs_to_tenant(&division)
    );
    println!(
        "  Team ARN belongs to Root: {}",
        team_arn.belongs_to_tenant(&root)
    );
    println!(
        "  Division ARN belongs to Team: {}",
        division_arn.belongs_to_tenant(&team)
    );
    println!();

    Ok(())
}

/// Demonstrates ARN introspection and helper methods.
#[allow(clippy::result_large_err)]
fn arn_introspection() -> Result<()> {
    println!("## 7. ARN Introspection\n");

    let arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_hierarchy(vec!["acme", "engineering", "backend"])
        .wami_instance("prod-001")
        .cloud_provider("aws", "223344556677")
        .resource("role", "backend-deploy-role")
        .build()?;

    println!("ARN: {}\n", arn);

    println!("Service Information:");
    println!("  Service: {}", arn.service);
    println!("  Service string: {}\n", arn.service.as_str());

    println!("Tenant Information:");
    println!("  Full tenant path: {}", arn.full_tenant_path());
    println!("  Primary (root) tenant: {}", arn.primary_tenant().unwrap());
    println!("  Leaf tenant: {}", arn.leaf_tenant().unwrap());
    println!("  Tenant depth: {}\n", arn.tenant_path.depth());

    println!("Instance Information:");
    println!("  WAMI instance ID: {}\n", arn.wami_instance_id);

    println!("Cloud Information:");
    println!("  Cloud synced: {}", arn.is_cloud_synced());
    if let Some(provider) = arn.provider() {
        println!("  Provider: {}", provider);
        if let Some(ref mapping) = arn.cloud_mapping {
            println!("  Account ID: {}", mapping.account_id);
        }
    }
    println!();

    println!("Resource Information:");
    println!("  Resource type: {}", arn.resource_type());
    println!("  Resource ID: {}", arn.resource_id());
    println!("  Resource path: {}\n", arn.resource.as_path());

    println!("ARN Components:");
    println!("  Prefix: {}", arn.prefix());
    println!("  Full ARN: {}\n", arn);

    Ok(())
}
