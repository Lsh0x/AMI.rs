//! Role Builder Functions

use super::model::Role;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::provider::{CloudProvider, ResourceType};
use chrono::Utc;
use uuid::Uuid;

/// Build a new Role with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_role(
    role_name: String,
    assume_role_policy_document: String,
    path: Option<String>,
    description: Option<String>,
    max_session_duration: Option<i32>,
    context: &WamiContext,
) -> Result<Role> {
    let role_id = Uuid::new_v4().to_string();
    let path = path.unwrap_or_else(|| "/".to_string());

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("role", &role_id)
        .build()?;

    // Generate AWS-compatible ARN (for backward compatibility)
    let arn = format!(
        "arn:aws:iam::{}:role{}{}",
        context.instance_id(),
        if path == "/" { "" } else { &path },
        role_name
    );

    Ok(Role {
        role_name,
        role_id,
        arn,
        path,
        create_date: Utc::now(),
        assume_role_policy_document,
        description,
        max_session_duration,
        permissions_boundary: None,
        tags: vec![],
        wami_arn,
        providers: Vec::new(),
        tenant_id: None,
    })
}

/// Build a new Role with provider-specific identifiers (legacy)
#[deprecated(note = "Use build_role with WamiContext instead")]
pub fn build_role_legacy(
    role_name: String,
    assume_role_policy_document: String,
    path: Option<String>,
    description: Option<String>,
    max_session_duration: Option<i32>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Role {
    let role_id = provider.generate_resource_id(ResourceType::Role);
    let path = path.unwrap_or_else(|| "/".to_string());
    let arn =
        provider.generate_resource_identifier(ResourceType::Role, account_id, &path, &role_name);
    let wami_arn_str =
        provider.generate_wami_arn(ResourceType::Role, account_id, &path, &role_name);

    // Parse the wami_arn string to WamiArn
    let wami_arn = wami_arn_str.parse().unwrap_or_else(|_| {
        // Fallback: create a basic ARN
        WamiArn::builder()
            .service(Service::Iam)
            .tenant("default")
            .wami_instance(account_id)
            .resource("role", &role_id)
            .build()
            .expect("Failed to build fallback ARN")
    });

    Role {
        role_name,
        role_id,
        arn,
        path,
        create_date: Utc::now(),
        assume_role_policy_document,
        description,
        max_session_duration,
        permissions_boundary: None,
        tags: vec![],
        wami_arn,
        providers: Vec::new(),
        tenant_id: None,
    }
}

/// Update role's assume role policy (pure transformation)
pub fn update_assume_role_policy(mut role: Role, new_policy: String) -> Role {
    role.assume_role_policy_document = new_policy;
    role
}

/// Update role's description (pure transformation)
pub fn update_description(mut role: Role, description: Option<String>) -> Role {
    role.description = description;
    role
}

/// Update role's max session duration (pure transformation)
pub fn update_max_session_duration(mut role: Role, duration: i32) -> Role {
    role.max_session_duration = Some(duration);
    role
}

/// Set role's permissions boundary (pure transformation)
pub fn set_permissions_boundary(mut role: Role, boundary_arn: String) -> Role {
    role.permissions_boundary = Some(boundary_arn);
    role
}

/// Clear role's permissions boundary (pure transformation)
pub fn clear_permissions_boundary(mut role: Role) -> Role {
    role.permissions_boundary = None;
    role
}

/// Add tags to role (pure transformation)
pub fn add_tags(mut role: Role, tags: Vec<crate::types::Tag>) -> Role {
    for tag in tags {
        if !role.tags.iter().any(|t| t.key == tag.key) {
            role.tags.push(tag);
        }
    }
    role
}

/// Set tenant ID (pure transformation)
pub fn set_tenant_id(mut role: Role, tenant_id: crate::wami::tenant::TenantId) -> Role {
    role.tenant_id = Some(tenant_id);
    role
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::TenantPath;
    use crate::types::Tag;
    use crate::wami::tenant::TenantId;

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:iam:test:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single("test"))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    fn test_trust_policy() -> String {
        r#"{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"ec2.amazonaws.com"},"Action":"sts:AssumeRole"}]}"#.to_string()
    }

    #[test]
    fn test_build_role_minimal() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        assert_eq!(role.role_name, "test-role");
        assert_eq!(role.path, "/");
        assert!(!role.role_id.is_empty());
        assert!(role.arn.contains("test-role"));
        assert!(role.description.is_none());
        assert!(role.max_session_duration.is_none());
        assert!(role.permissions_boundary.is_none());
        assert_eq!(role.tags.len(), 0);
    }

    #[test]
    fn test_build_role_with_all_options() {
        let context = test_context();
        let role = build_role(
            "admin-role".to_string(),
            test_trust_policy(),
            Some("/admin/".to_string()),
            Some("Administrator role".to_string()),
            Some(7200),
            &context,
        )
        .unwrap();

        assert_eq!(role.role_name, "admin-role");
        assert_eq!(role.path, "/admin/");
        assert_eq!(role.description, Some("Administrator role".to_string()));
        assert_eq!(role.max_session_duration, Some(7200));
    }

    #[test]
    fn test_update_assume_role_policy() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        let new_policy = r#"{"Version":"2012-10-17"}"#.to_string();
        let updated = update_assume_role_policy(role, new_policy.clone());

        assert_eq!(updated.assume_role_policy_document, new_policy);
    }

    #[test]
    fn test_update_description() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        let updated = update_description(role, Some("New description".to_string()));
        assert_eq!(updated.description, Some("New description".to_string()));

        let updated = update_description(updated, None);
        assert_eq!(updated.description, None);
    }

    #[test]
    fn test_update_max_session_duration() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        let updated = update_max_session_duration(role, 3600);
        assert_eq!(updated.max_session_duration, Some(3600));
    }

    #[test]
    fn test_set_permissions_boundary() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        let boundary = "arn:aws:iam::123:policy/boundary".to_string();
        let updated = set_permissions_boundary(role, boundary.clone());

        assert_eq!(updated.permissions_boundary, Some(boundary));
    }

    #[test]
    fn test_clear_permissions_boundary() {
        let context = test_context();
        let mut role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        role.permissions_boundary = Some("arn:aws:iam::123:policy/boundary".to_string());
        let updated = clear_permissions_boundary(role);

        assert!(updated.permissions_boundary.is_none());
    }

    #[test]
    fn test_add_tags() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        let tags = vec![
            Tag {
                key: "Env".to_string(),
                value: "Prod".to_string(),
            },
            Tag {
                key: "Team".to_string(),
                value: "Platform".to_string(),
            },
        ];

        let updated = add_tags(role, tags);
        assert_eq!(updated.tags.len(), 2);
    }

    #[test]
    fn test_add_tags_no_duplicates() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        let tags1 = vec![Tag {
            key: "Env".to_string(),
            value: "Prod".to_string(),
        }];
        let tags2 = vec![Tag {
            key: "Env".to_string(),
            value: "Dev".to_string(),
        }];

        let updated = add_tags(role, tags1);
        let updated = add_tags(updated, tags2);

        assert_eq!(updated.tags.len(), 1);
        assert_eq!(updated.tags[0].value, "Prod");
    }

    #[test]
    fn test_set_tenant_id() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        let tenant_id = TenantId::new("acme");
        let updated = set_tenant_id(role, tenant_id.clone());

        assert_eq!(updated.tenant_id, Some(tenant_id));
    }

    #[test]
    fn test_role_immutability() {
        let context = test_context();
        let role = build_role(
            "test-role".to_string(),
            test_trust_policy(),
            None,
            None,
            None,
            &context,
        )
        .unwrap();

        let original_name = role.role_name.clone();
        let _ = update_max_session_duration(role.clone(), 9999);

        assert_eq!(role.role_name, original_name);
        assert_eq!(role.max_session_duration, None);
    }
}
