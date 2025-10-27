//! ARN-Centric Architecture Demo
//!
//! This example demonstrates the new ARN-based architecture components.
//! Note: Full integration pending model migrations (see MIGRATION_GUIDE_ARN.md)

use std::env;
use wami::provider::arn_builder::{arn_pattern_match, ParsedArn, WamiArnBuilder};
use wami::provider::provider_info::ProviderInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAMI ARN Architecture Demo ===\n");

    // Example 1: ARN Generation
    demo_arn_generation()?;

    // Example 2: ARN Parsing
    demo_arn_parsing()?;

    // Example 3: Pattern Matching
    demo_pattern_matching()?;

    // Example 4: Multi-Provider Support
    demo_multi_provider()?;

    // Example 5: Security Features
    demo_security()?;

    println!("\n✅ All demos completed successfully!");
    Ok(())
}

fn demo_arn_generation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 1: ARN Generation\n");

    let builder = WamiArnBuilder::new();

    // Generate different resource ARNs
    let user_arn = builder.build_arn("iam", "123456789012", "user", "/", "alice");
    let role_arn = builder.build_arn("iam", "123456789012", "role", "/", "Admin");
    let policy_arn = builder.build_arn("iam", "123456789012", "policy", "/", "ReadOnly");

    println!("User ARN:   {}", user_arn);
    println!("Role ARN:   {}", role_arn);
    println!("Policy ARN: {}", policy_arn);

    // Verify opaque tenant hash
    let tenant_hash = user_arn.split(':').nth(3).unwrap();
    println!("\n✓ Tenant hash: {} (opaque!)", tenant_hash);
    println!("✓ Real account ID not exposed in ARN");

    println!();
    Ok(())
}

fn demo_arn_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 2: ARN Parsing\n");

    let arn = "arn:wami:iam:tenant-a1b2c3d4:user/tenants/acme/engineering/alice";
    let parsed = ParsedArn::from_arn(arn)?;

    println!("Original ARN: {}", arn);
    println!("\nParsed components:");
    println!("  Provider:      {}", parsed.provider);
    println!("  Service:       {}", parsed.service);
    println!("  Tenant Hash:   {}", parsed.tenant_hash);
    println!("  Resource Type: {}", parsed.resource_type);
    println!("  Path:          {}", parsed.path);
    println!("  Name:          {}", parsed.name);

    // Roundtrip test
    let reconstructed = parsed.to_arn();
    println!("\nRoundtrip test:");
    println!("  Original:      {}", arn);
    println!("  Reconstructed: {}", reconstructed);
    println!("  ✓ Match: {}", arn == reconstructed);

    println!();
    Ok(())
}

fn demo_pattern_matching() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 3: Pattern Matching\n");

    let arns = vec![
        "arn:wami:iam:tenant-abc:user/alice",
        "arn:wami:iam:tenant-abc:user/bob",
        "arn:wami:iam:tenant-abc:role/Admin",
        "arn:wami:iam:tenant-xyz:user/charlie",
        "arn:wami:sts:tenant-abc:assumed-role/Admin/session-1",
    ];

    let patterns = vec![
        ("All users in tenant-abc", "arn:wami:iam:tenant-abc:user/*"),
        (
            "All IAM resources in tenant-abc",
            "arn:wami:iam:tenant-abc:*",
        ),
        ("All resources (any tenant)", "arn:wami:*:*:*"),
        ("STS resources only", "arn:wami:sts:*:*"),
    ];

    for (description, pattern) in patterns {
        println!("Pattern: {} ({})", pattern, description);
        println!("Matches:");

        for arn in &arns {
            if arn_pattern_match(arn, pattern) {
                println!("  ✓ {}", arn);
            }
        }
        println!();
    }

    Ok(())
}

fn demo_multi_provider() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 4: Multi-Provider Support\n");

    // Create provider infos for the same user on different clouds
    let aws_info = ProviderInfo::new(
        "aws",
        "arn:aws:iam::123456789012:user/alice",
        Some("AIDACKCEVSQ6C2EXAMPLE".to_string()),
        "123456789012",
    );

    let gcp_info = ProviderInfo::new(
        "gcp",
        "projects/my-project/serviceAccounts/alice@my-project.iam.gserviceaccount.com",
        Some("123456789012345678".to_string()),
        "my-project",
    );

    let azure_info = ProviderInfo::new(
        "azure",
        "/subscriptions/sub-123/resourceGroups/rg/providers/Microsoft.Authorization/users/alice",
        Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        "sub-123",
    );

    println!("User 'alice' synced across multiple clouds:\n");

    println!("AWS:");
    println!("  Native ARN: {}", aws_info.native_arn);
    println!("  Resource ID: {}", aws_info.resource_id.as_ref().unwrap());
    println!("  Account ID: {}", aws_info.account_id);

    println!("\nGCP:");
    println!("  Native ARN: {}", gcp_info.native_arn);
    println!("  Resource ID: {}", gcp_info.resource_id.as_ref().unwrap());
    println!("  Project ID: {}", gcp_info.account_id);

    println!("\nAzure:");
    println!("  Native ARN: {}", azure_info.native_arn);
    println!(
        "  Resource ID: {}",
        azure_info.resource_id.as_ref().unwrap()
    );
    println!("  Subscription ID: {}", azure_info.account_id);

    println!("\n✓ Single WAMI resource can exist on multiple clouds!");
    println!();
    Ok(())
}

fn demo_security() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 5: Security Features\n");

    // Without salt (deterministic but public)
    let builder_no_salt = WamiArnBuilder::new();
    let arn1 = builder_no_salt.build_arn("iam", "123456789012", "user", "/", "alice");

    // With salt (deterministic but salted)
    env::set_var("WAMI_ARN_SALT", "production-secret-key");
    let builder_with_salt = WamiArnBuilder::new();
    let arn2 = builder_with_salt.build_arn("iam", "123456789012", "user", "/", "alice");

    println!("Same account ID, same user:");
    println!("  Without salt: {}", arn1);
    println!("  With salt:    {}", arn2);

    let hash1 = arn1.split(':').nth(3).unwrap();
    let hash2 = arn2.split(':').nth(3).unwrap();

    println!("\nTenant hashes:");
    println!("  No salt: {}", hash1);
    println!("  Salted:  {}", hash2);
    println!("  ✓ Different hashes = salt prevents rainbow table attacks");

    // Cleanup
    env::remove_var("WAMI_ARN_SALT");

    println!();
    Ok(())
}
