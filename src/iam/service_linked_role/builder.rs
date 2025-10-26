//! Service Linked Role Builder

use crate::iam::Role;
use crate::provider::{CloudProvider, ResourceType};
use chrono::Utc;

/// Build a new service-linked Role
pub fn build_service_linked_role(
    aws_service_name: &str,
    description: Option<String>,
    custom_suffix: Option<&str>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Role {
    // Generate role name using provider
    let role_name = if custom_suffix.is_none() {
        provider.generate_service_linked_role_name(aws_service_name, None)
    } else {
        // Extract service name and format with custom suffix
        let service_name = aws_service_name.split('.').next().unwrap_or("");
        let service_name_pascal = service_name
            .split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<String>();

        format!(
            "AWSServiceRoleFor{}_{}",
            service_name_pascal,
            custom_suffix.unwrap()
        )
    };

    // Service-linked roles have a specific path pattern
    let path = provider.generate_service_linked_role_path(aws_service_name);

    // Generate the assume role policy document
    let assume_role_policy_document = serde_json::json!({
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": {
                "Service": aws_service_name
            },
            "Action": "sts:AssumeRole"
        }]
    })
    .to_string();

    let role_id = provider.generate_resource_id(ResourceType::ServiceLinkedRole);
    let arn = provider.generate_resource_identifier(
        ResourceType::ServiceLinkedRole,
        account_id,
        &path,
        &role_name,
    );
    let wami_arn = provider.generate_wami_arn(
        ResourceType::ServiceLinkedRole,
        account_id,
        &path,
        &role_name,
    );

    Role {
        role_name,
        role_id,
        arn,
        path,
        create_date: Utc::now(),
        assume_role_policy_document,
        description,
        max_session_duration: Some(3600),
        permissions_boundary: None,
        tags: vec![],
        wami_arn,
        providers: Vec::new(),
    }
}
