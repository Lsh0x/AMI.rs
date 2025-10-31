//! Group Builder Functions

use super::model::Group;
use crate::arn::{Service, WamiArn};
use crate::context::WamiContext;
use crate::error::Result;
use crate::provider::{CloudProvider, ResourceType};
use chrono::Utc;
use uuid::Uuid;

/// Build a new Group with context-based identifiers
#[allow(clippy::result_large_err)]
pub fn build_group(
    group_name: String,
    path: Option<String>,
    context: &WamiContext,
) -> Result<Group> {
    let group_id = Uuid::new_v4().to_string();
    let path = path.unwrap_or_else(|| "/".to_string());

    // Build WAMI ARN using context
    let wami_arn = WamiArn::builder()
        .service(Service::Iam)
        .tenant_path(context.tenant_path().clone())
        .wami_instance(context.instance_id())
        .resource("group", &group_id)
        .build()?;

    // Generate AWS-compatible ARN (for backward compatibility)
    let arn = format!(
        "arn:aws:iam::{}:group{}{}",
        context.instance_id(),
        if path == "/" { "" } else { &path },
        group_name
    );

    Ok(Group {
        group_name,
        group_id,
        arn,
        path,
        create_date: Utc::now(),
        wami_arn,
        providers: Vec::new(),
        tenant_id: None,
        tags: vec![],
    })
}

/// Build a new Group with provider-specific identifiers (legacy)
#[deprecated(note = "Use build_group with WamiContext instead")]
pub fn build_group_legacy(
    group_name: String,
    path: Option<String>,
    provider: &dyn CloudProvider,
    account_id: &str,
) -> Group {
    let group_id = provider.generate_resource_id(ResourceType::Group);
    let path = path.unwrap_or_else(|| "/".to_string());
    let arn =
        provider.generate_resource_identifier(ResourceType::Group, account_id, &path, &group_name);
    let wami_arn_str =
        provider.generate_wami_arn(ResourceType::Group, account_id, &path, &group_name);

    // Parse the wami_arn string to WamiArn
    let wami_arn = wami_arn_str.parse().unwrap_or_else(|_| {
        // Fallback: create a basic ARN
        WamiArn::builder()
            .service(Service::Iam)
            .tenant(12345678u64) // Test tenant ID
            .wami_instance(account_id)
            .resource("group", &group_id)
            .build()
            .expect("Failed to build fallback ARN")
    });

    Group {
        group_name,
        group_id,
        arn,
        path,
        create_date: Utc::now(),
        wami_arn,
        providers: Vec::new(),
        tenant_id: None,
        tags: vec![],
    }
}

/// Update group's name (pure transformation)
pub fn update_group_name(mut group: Group, new_name: String) -> Group {
    group.group_name = new_name;
    group
}

/// Update group's path (pure transformation)
pub fn update_group_path(mut group: Group, new_path: String) -> Group {
    group.path = new_path;
    group
}

/// Set group's tenant ID (pure transformation)
pub fn set_tenant_id(mut group: Group, tenant_id: crate::wami::tenant::TenantId) -> Group {
    group.tenant_id = Some(tenant_id);
    group
}

/// Add provider to group (pure transformation)
pub fn add_provider(mut group: Group, provider_config: crate::provider::ProviderConfig) -> Group {
    if !group
        .providers
        .iter()
        .any(|p| p.provider_name == provider_config.provider_name)
    {
        group.providers.push(provider_config);
    }
    group
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arn::TenantPath;
    use crate::wami::tenant::TenantId;

    fn test_context() -> WamiContext {
        let arn: WamiArn = "arn:wami:.*:12345678:wami:123456789012:user/test"
            .parse()
            .unwrap();
        WamiContext::builder()
            .instance_id("123456789012")
            .tenant_path(TenantPath::single(12345678))
            .caller_arn(arn)
            .is_root(false)
            .build()
            .unwrap()
    }

    #[test]
    fn test_build_group() {
        let context = test_context();
        let group = build_group("admins".to_string(), None, &context).unwrap();

        assert_eq!(group.group_name, "admins");
        assert_eq!(group.path, "/");
        assert!(!group.group_id.is_empty());
        assert!(group.arn.contains("admins"));
        // WamiArn uses group_id (UUID), not group_name - verify structure instead
        assert_eq!(group.wami_arn.resource.resource_type, "group");
        assert_eq!(group.wami_arn.resource.resource_id, group.group_id);
        assert!(group.tenant_id.is_none());
        assert_eq!(group.providers.len(), 0);
    }

    #[test]
    fn test_build_group_with_path() {
        let context = test_context();
        let group = build_group(
            "developers".to_string(),
            Some("/engineering/".to_string()),
            &context,
        )
        .unwrap();

        assert_eq!(group.group_name, "developers");
        assert_eq!(group.path, "/engineering/");
    }

    #[test]
    fn test_update_group_name() {
        let context = test_context();
        let group = build_group("old-name".to_string(), None, &context).unwrap();
        let updated = update_group_name(group, "new-name".to_string());

        assert_eq!(updated.group_name, "new-name");
    }

    #[test]
    fn test_update_group_path() {
        let context = test_context();
        let group = build_group("admins".to_string(), None, &context).unwrap();
        let updated = update_group_path(group, "/admin/".to_string());

        assert_eq!(updated.path, "/admin/");
    }

    #[test]
    fn test_set_tenant_id() {
        let context = test_context();
        let group = build_group("admins".to_string(), None, &context).unwrap();
        let tenant_id = TenantId::root(); // Test with root tenant

        let updated = set_tenant_id(group, tenant_id.clone());
        assert_eq!(updated.tenant_id, Some(tenant_id));
    }

    #[test]
    fn test_add_provider() {
        let context = test_context();
        let group = build_group("admins".to_string(), None, &context).unwrap();

        let provider_config = crate::provider::ProviderConfig {
            provider_name: "aws".to_string(),
            account_id: "123456789012".to_string(),
            native_arn: "arn:aws:iam::123456789012:group/admins".to_string(),
            synced_at: chrono::Utc::now(),
            tenant_id: None,
        };

        let updated = add_provider(group, provider_config);
        assert_eq!(updated.providers.len(), 1);
        assert_eq!(updated.providers[0].provider_name, "aws");
    }

    #[test]
    fn test_add_provider_no_duplicates() {
        let context = test_context();
        let group = build_group("admins".to_string(), None, &context).unwrap();

        let provider_config = crate::provider::ProviderConfig {
            provider_name: "aws".to_string(),
            account_id: "123456789012".to_string(),
            native_arn: "arn:aws:iam::123456789012:group/admins".to_string(),
            synced_at: chrono::Utc::now(),
            tenant_id: None,
        };

        let updated = add_provider(group, provider_config.clone());
        let updated = add_provider(updated, provider_config);

        assert_eq!(updated.providers.len(), 1);
    }

    #[test]
    fn test_build_group_immutable() {
        let context = test_context();
        let group = build_group("test".to_string(), None, &context).unwrap();
        let group_name = group.group_name.clone();

        let _ = update_group_name(group.clone(), "changed".to_string());

        // Original should be unchanged
        assert_eq!(group.group_name, group_name);
    }
}
