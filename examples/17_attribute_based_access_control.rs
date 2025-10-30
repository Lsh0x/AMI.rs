//! Attribute-Based Access Control (ABAC)
//!
//! This example demonstrates:
//! - Tagging resources with attributes
//! - Creating tag-based policies
//! - Dynamic access control based on attributes
//!
//! Scenario: Access control based on department and project tags.
//!
//! Run with: `cargo run --example 17_attribute_based_access_control`

use std::sync::{Arc, RwLock};
use wami::provider::AwsProvider;
use wami::service::{PolicyService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::types::Tag;
use wami::wami::identity::user::requests::CreateUserRequest;
use wami::wami::policies::policy::requests::CreatePolicyRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Attribute-Based Access Control (ABAC) ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    let _provider = Arc::new(AwsProvider::new());
    let account_id = "123456789012";

    let user_service = UserService::new(store.clone(), account_id.to_string());
    let policy_service = PolicyService::new(store.clone(), account_id.to_string());

    // === CREATE TAGGED USERS ===
    println!("Step 1: Creating users with department/project tags...\n");

    user_service
        .create_user(CreateUserRequest {
            user_name: "alice".to_string(),
            path: Some("/".to_string()),
            permissions_boundary: None,
            tags: Some(vec![
                Tag {
                    key: "Department".to_string(),
                    value: "Engineering".to_string(),
                },
                Tag {
                    key: "Project".to_string(),
                    value: "ProjectA".to_string(),
                },
            ]),
        })
        .await?;
    println!("✓ Created alice (Engineering, ProjectA)");

    user_service
        .create_user(CreateUserRequest {
            user_name: "bob".to_string(),
            path: Some("/".to_string()),
            permissions_boundary: None,
            tags: Some(vec![
                Tag {
                    key: "Department".to_string(),
                    value: "Engineering".to_string(),
                },
                Tag {
                    key: "Project".to_string(),
                    value: "ProjectB".to_string(),
                },
            ]),
        })
        .await?;
    println!("✓ Created bob (Engineering, ProjectB)");

    user_service
        .create_user(CreateUserRequest {
            user_name: "charlie".to_string(),
            path: Some("/".to_string()),
            permissions_boundary: None,
            tags: Some(vec![
                Tag {
                    key: "Department".to_string(),
                    value: "Sales".to_string(),
                },
                Tag {
                    key: "Project".to_string(),
                    value: "ProjectC".to_string(),
                },
            ]),
        })
        .await?;
    println!("✓ Created charlie (Sales, ProjectC)");

    // === CREATE ABAC POLICY ===
    println!("\n\nStep 2: Creating ABAC policy...\n");

    let abac_policy_doc = r#"{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "s3:*",
    "Resource": "arn:aws:s3:::${aws:PrincipalTag/Project}/*",
    "Condition": {
      "StringEquals": {
        "s3:ExistingObjectTag/Department": "${aws:PrincipalTag/Department}"
      }
    }
  }]
}"#;

    policy_service
        .create_policy(CreatePolicyRequest {
            policy_name: "ABACPolicy".to_string(),
            path: Some("/abac/".to_string()),
            policy_document: abac_policy_doc.to_string(),
            description: Some("ABAC policy using tags".to_string()),
            tags: None,
        })
        .await?;
    println!("✓ Created ABAC policy with tag-based conditions");
    println!("  - Resource access based on PrincipalTag/Project");
    println!("  - Object access requires matching Department tag");

    println!("\n\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- ABAC uses tags for dynamic access control");
    println!("- Policies reference ${{aws:PrincipalTag/Key}}");
    println!("- Scales better than RBAC for large organizations");

    Ok(())
}
