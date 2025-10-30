//! Policy Evaluation Simulation
//!
//! This example demonstrates:
//! - Using EvaluationService to simulate policy evaluation
//! - Testing permissions before deployment
//! - Understanding policy decisions
//!
//! Scenario: Simulating whether alice can perform specific actions.
//!
//! Run with: `cargo run --example 15_policy_evaluation_simulation`

use std::sync::{Arc, RwLock};
use wami::provider::AwsProvider;
use wami::service::{EvaluationService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::requests::CreateUserRequest;
use wami::wami::policies::evaluation::requests::SimulateCustomPolicyRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Policy Evaluation Simulation ===\n");

    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));
    let _provider = Arc::new(AwsProvider::new());
    let account_id = "123456789012";

    let eval_service = EvaluationService::new(store.clone(), account_id.to_string());
    let user_service = UserService::new(store.clone(), account_id.to_string());

    // Create user
    println!("Step 1: Creating user...\n");
    let req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/".to_string()),
        permissions_boundary: None,
        tags: None,
    };
    let alice = user_service.create_user(req).await?;
    println!("✓ Created alice: {}", alice.arn);

    // === SIMULATE POLICY ===
    println!("\n\nStep 2: Simulating custom policy...\n");

    let policy_doc = r#"{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["s3:GetObject", "s3:PutObject"],
    "Resource": "arn:aws:s3:::my-bucket/*"
  }]
}"#;

    // Test allowed action
    let sim_req = SimulateCustomPolicyRequest {
        policy_input_list: vec![policy_doc.to_string()],
        action_names: vec!["s3:GetObject".to_string()],
        resource_arns: Some(vec!["arn:aws:s3:::my-bucket/file.txt".to_string()]),
        context_entries: None,
    };

    let result = eval_service.simulate_custom_policy(sim_req).await?;
    println!("✓ Simulation: s3:GetObject on my-bucket/file.txt");
    println!("  Decision: {}", result.evaluation_results[0].eval_decision);

    // Test denied action
    let denied_req = SimulateCustomPolicyRequest {
        policy_input_list: vec![policy_doc.to_string()],
        action_names: vec!["s3:DeleteObject".to_string()],
        resource_arns: Some(vec!["arn:aws:s3:::my-bucket/file.txt".to_string()]),
        context_entries: None,
    };

    let denied_result = eval_service.simulate_custom_policy(denied_req).await?;
    println!("\n✓ Simulation: s3:DeleteObject on my-bucket/file.txt");
    println!(
        "  Decision: {}",
        denied_result.evaluation_results[0].eval_decision
    );

    println!("\n✅ Example completed successfully!");
    println!("Key takeaways:");
    println!("- Policy simulation helps test before deployment");
    println!("- Simulate custom policies or principal policies");
    println!("- Understand allow/deny decisions");
    println!("- Identify missing permissions");

    Ok(())
}
