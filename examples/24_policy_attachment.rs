//! Example 24: Policy Attachment
//!
//! This example demonstrates how to attach managed policies and inline policies
//! to users, groups, and roles.

use std::sync::{Arc, RwLock};
use wami::arn::{TenantPath, WamiArn};
use wami::context::WamiContext;
use wami::service::{AttachmentService, InlinePolicyService, PolicyService, UserService};
use wami::store::memory::InMemoryWamiStore;
use wami::wami::identity::user::CreateUserRequest;
use wami::wami::policies::attachment::{AttachUserPolicyRequest, ListAttachedUserPoliciesRequest};
use wami::wami::policies::inline::{
    GetUserPolicyRequest, ListUserPoliciesRequest, PutUserPolicyRequest,
};
use wami::wami::policies::policy::CreatePolicyRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Initialize store
    let store = Arc::new(RwLock::new(InMemoryWamiStore::default()));

    // Create context
    let context = WamiContext::builder()
        .instance_id("123456789012")
        .tenant_path(TenantPath::single("root"))
        .caller_arn(
            WamiArn::builder()
                .service(wami::arn::Service::Iam)
                .tenant_path(TenantPath::single("root"))
                .wami_instance("123456789012")
                .resource("user", "admin")
                .build()?,
        )
        .is_root(false)
        .build()?;

    println!("=== Policy Attachment Example ===\n");

    // Step 1: Create a user
    println!("1. Creating user 'alice'...");
    let user_service = UserService::new(store.clone());
    let create_user_req = CreateUserRequest {
        user_name: "alice".to_string(),
        path: Some("/".to_string()),
        tags: Some(vec![]),
        permissions_boundary: None,
    };
    let user = user_service.create_user(&context, create_user_req).await?;
    println!("   Created user: {} (ARN: {})\n", user.user_name, user.arn);

    // Step 2: Create a managed policy
    println!("2. Creating managed policy 'S3ReadOnly'...");
    let policy_service = PolicyService::new(store.clone());
    let policy_doc = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": ["s3:GetObject", "s3:ListBucket"],
            "Resource": "*"
        }]
    }"#;

    let create_policy_req = CreatePolicyRequest {
        policy_name: "S3ReadOnly".to_string(),
        path: Some("/".to_string()),
        policy_document: policy_doc.to_string(),
        description: Some("Read-only access to S3".to_string()),
        tags: Some(vec![]),
    };
    let policy = policy_service
        .create_policy(&context, create_policy_req)
        .await?;
    println!(
        "   Created policy: {} (ARN: {})\n",
        policy.policy_name, policy.arn
    );

    // Step 3: Attach the managed policy to the user
    println!("3. Attaching managed policy to user...");
    let attachment_service = AttachmentService::new(store.clone());
    let attach_req = AttachUserPolicyRequest {
        user_name: "alice".to_string(),
        policy_arn: policy.arn.clone(),
    };
    let attach_resp = attachment_service.attach_user_policy(attach_req).await?;
    println!("   {}\n", attach_resp.message);

    // Step 4: List attached policies
    println!("4. Listing attached policies for user 'alice'...");
    let list_req = ListAttachedUserPoliciesRequest {
        user_name: "alice".to_string(),
    };
    let list_resp = attachment_service
        .list_attached_user_policies(list_req)
        .await?;
    println!(
        "   Found {} attached policies:",
        list_resp.attached_policies.len()
    );
    for p in &list_resp.attached_policies {
        println!("   - {} ({})", p.policy_name, p.policy_arn);
    }
    println!();

    // Step 5: Add an inline policy
    println!("5. Adding inline policy 'DenyDelete' to user...");
    let inline_service = InlinePolicyService::new(store.clone());
    let inline_doc = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Deny",
            "Action": ["*:Delete*"],
            "Resource": "*"
        }]
    }"#;

    let put_inline_req = PutUserPolicyRequest {
        user_name: "alice".to_string(),
        policy_name: "DenyDelete".to_string(),
        policy_document: inline_doc.to_string(),
    };
    let put_resp = inline_service.put_user_policy(put_inline_req).await?;
    println!("   {}\n", put_resp.message);

    // Step 6: List inline policies
    println!("6. Listing inline policies for user 'alice'...");
    let list_inline_req = ListUserPoliciesRequest {
        user_name: "alice".to_string(),
    };
    let list_inline_resp = inline_service.list_user_policies(list_inline_req).await?;
    println!(
        "   Found {} inline policies:",
        list_inline_resp.policy_names.len()
    );
    for name in &list_inline_resp.policy_names {
        println!("   - {}", name);
    }
    println!();

    // Step 7: Get inline policy content
    println!("7. Getting inline policy 'DenyDelete'...");
    let get_inline_req = GetUserPolicyRequest {
        user_name: "alice".to_string(),
        policy_name: "DenyDelete".to_string(),
    };
    let get_resp = inline_service.get_user_policy(get_inline_req).await?;
    println!("   Policy document:\n{}\n", get_resp.policy_document);

    println!("=== Summary ===");
    println!("User 'alice' now has:");
    println!(
        "- {} managed policy attached (S3ReadOnly)",
        list_resp.attached_policies.len()
    );
    println!(
        "- {} inline policy (DenyDelete)",
        list_inline_resp.policy_names.len()
    );
    println!("\nThis grants alice read access to S3 but denies all delete operations.");

    Ok(())
}
