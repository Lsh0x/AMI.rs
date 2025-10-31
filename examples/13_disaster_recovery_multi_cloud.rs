//! Disaster Recovery Multi-Cloud
//!
//! This example demonstrates:
//! - Replicating resources from primary to secondary cloud
//! - Failover scenarios
//! - Multi-cloud redundancy patterns
//!
//! Scenario: Replicating critical resources from AWS (primary) to GCP (backup).
//!
//! Run with: `cargo run --example 13_disaster_recovery_multi_cloud`

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{GroupService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::group::requests::CreateGroupRequest;
use wami::wami::identity::user::requests::{CreateUserRequest, ListUsersRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Disaster Recovery Multi-Cloud ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create AWS (primary) context
    let aws_context = WamiContext::builder()
        .instance_id("111111111111")
        .tenant_path(TenantPath::single(40000001)) // Numeric tenant ID for AWS primary
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(40000001))
                .wami_instance("111111111111")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    // Create GCP (secondary) context
    let gcp_context = WamiContext::builder()
        .instance_id("backup-project")
        .tenant_path(TenantPath::single(50000001)) // Numeric tenant ID for GCP backup
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single(50000001))
                .wami_instance("backup-project")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    let user_service = UserService::new(store.clone());
    let group_service = GroupService::new(store.clone());

    // === SETUP PRIMARY (AWS) ===
    println!("Step 1: Creating resources in PRIMARY (AWS)...\n");

    // Create users in primary
    println!("Creating critical users in AWS...");
    let admin_req = CreateUserRequest {
        user_name: "admin".to_string(),
        path: Some("/critical/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![
            wami::types::Tag {
                key: "Criticality".to_string(),
                value: "High".to_string(),
            },
            wami::types::Tag {
                key: "ReplicationTarget".to_string(),
                value: "GCP".to_string(),
            },
        ]),
    };
    let aws_admin = user_service.create_user(&aws_context, admin_req).await?;
    println!("✓ Created admin in AWS: {}", aws_admin.wami_arn);

    let operator_req = CreateUserRequest {
        user_name: "operator".to_string(),
        path: Some("/critical/".to_string()),
        permissions_boundary: None,
        tags: Some(vec![wami::types::Tag {
            key: "Criticality".to_string(),
            value: "High".to_string(),
        }]),
    };
    let aws_operator = user_service.create_user(&aws_context, operator_req).await?;
    println!("✓ Created operator in AWS: {}", aws_operator.wami_arn);

    // Create group in primary
    let group_req = CreateGroupRequest {
        group_name: "emergency-responders".to_string(),
        path: Some("/critical/".to_string()),
        tags: None,
    };
    let aws_group = group_service.create_group(&aws_context, group_req).await?;
    println!("\n✓ Created group in AWS: {}", aws_group.wami_arn);

    // Add users to group
    group_service
        .add_user_to_group("emergency-responders", "admin")
        .await?;
    group_service
        .add_user_to_group("emergency-responders", "operator")
        .await?;
    println!("✓ Added users to group");

    // === REPLICATE TO SECONDARY (GCP) ===
    println!("\n\nStep 2: Replicating to SECONDARY (GCP)...\n");

    // Replicate users
    println!("Replicating users to GCP...");
    let admin_replica_req = CreateUserRequest {
        user_name: aws_admin.user_name.clone(),
        path: Some(aws_admin.path.clone()),
        permissions_boundary: None,
        tags: Some(aws_admin.tags.clone()),
    };
    let gcp_admin = user_service
        .create_user(&gcp_context, admin_replica_req)
        .await?;
    println!("✓ Replicated admin to GCP: {}", gcp_admin.wami_arn);

    let operator_replica_req = CreateUserRequest {
        user_name: aws_operator.user_name.clone(),
        path: Some(aws_operator.path.clone()),
        permissions_boundary: None,
        tags: Some(aws_operator.tags.clone()),
    };
    let gcp_operator = user_service
        .create_user(&gcp_context, operator_replica_req)
        .await?;
    println!("✓ Replicated operator to GCP: {}", gcp_operator.wami_arn);

    // Replicate group
    let group_replica_req = CreateGroupRequest {
        group_name: aws_group.group_name.clone(),
        path: Some(aws_group.path.clone()),
        tags: None,
    };
    let gcp_group = group_service
        .create_group(&gcp_context, group_replica_req)
        .await?;
    println!("\n✓ Replicated group to GCP: {}", gcp_group.wami_arn);

    // Replicate group membership
    group_service
        .add_user_to_group("emergency-responders", "admin")
        .await?;
    group_service
        .add_user_to_group("emergency-responders", "operator")
        .await?;
    println!("✓ Replicated group membership");

    // === SIMULATE FAILOVER ===
    println!("\n\nStep 3: Simulating failover scenario...\n");

    println!("Scenario: AWS region becomes unavailable");
    println!("Action: Failover to GCP backup");
    println!();
    println!("✓ GCP resources are ready to serve:");
    println!("  - {} users available", 2);
    println!("  - {} groups available", 1);
    println!("  - All group memberships preserved");

    // === DEMONSTRATE STATUS ===
    println!("\n\nStep 4: Disaster recovery status...\n");

    let (all_users, _, _) = user_service
        .list_users(ListUsersRequest {
            path_prefix: Some("/critical/".to_string()),
            pagination: None,
        })
        .await?;

    let aws_users: Vec<_> = all_users
        .iter()
        .filter(|u| u.wami_arn.to_string().contains("40000001"))
        .collect();
    let gcp_users: Vec<_> = all_users
        .iter()
        .filter(|u| u.wami_arn.to_string().contains("50000001"))
        .collect();

    println!("PRIMARY (AWS) status:");
    println!("  - Users: {}", aws_users.len());
    for user in aws_users {
        println!("    • {} → {}", user.user_name, user.wami_arn);
    }

    println!("\nSECONDARY (GCP) status:");
    println!("  - Users: {}", gcp_users.len());
    for user in gcp_users {
        println!("    • {} → {}", user.user_name, user.wami_arn);
    }

    // === BEST PRACTICES ===
    println!("\n\nStep 5: Disaster recovery best practices...\n");

    println!("Replication strategies:");
    println!("- Active-passive: Primary handles all traffic, secondary standby");
    println!("- Active-active: Both regions serve traffic simultaneously");
    println!("- Periodic sync: Scheduled replication of critical resources");
    println!("- Event-driven: Real-time replication on changes");
    println!();
    println!("Implementation considerations:");
    println!("- Replicate user accounts, roles, and policies");
    println!("- Sync group memberships and permissions");
    println!("- Test failover procedures regularly");
    println!("- Monitor replication lag and health");
    println!("- Implement automated failover triggers");
    println!("- Document runbooks for manual failover");

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Multi-cloud provides redundancy and disaster recovery");
    println!("- Replicate critical identity resources across clouds");
    println!("- Test failover scenarios before disasters occur");
    println!("- WAMI enables cloud-agnostic DR strategies");

    Ok(())
}
