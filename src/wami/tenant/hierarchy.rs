//! Tenant Hierarchy Utilities

use super::{Tenant, TenantId};

/// Represents a node in the tenant hierarchy tree
#[derive(Debug, Clone)]
pub struct TenantNode {
    /// The tenant at this node
    pub tenant: Tenant,
    /// Child tenant nodes
    pub children: Vec<TenantNode>,
}

impl TenantNode {
    /// Create a new tenant node
    pub fn new(tenant: Tenant) -> Self {
        Self {
            tenant,
            children: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: TenantNode) {
        self.children.push(child);
    }

    /// Get all descendant tenant IDs (excluding self)
    pub fn all_descendants(&self) -> Vec<TenantId> {
        let mut descendants = Vec::new();

        for child in &self.children {
            descendants.push(child.tenant.id.clone());
            descendants.extend(child.all_descendants());
        }

        descendants
    }

    /// Get total number of descendants
    pub fn descendant_count(&self) -> usize {
        let mut count = self.children.len();

        for child in &self.children {
            count += child.descendant_count();
        }

        count
    }

    /// Build a tree from a flat list of tenants
    pub fn build_tree(tenants: Vec<Tenant>, root_id: &TenantId) -> Option<Self> {
        let root_tenant = tenants.iter().find(|t| &t.id == root_id)?.clone();
        let mut root_node = Self::new(root_tenant);

        Self::add_children(&mut root_node, &tenants);

        Some(root_node)
    }

    fn add_children(node: &mut TenantNode, all_tenants: &[Tenant]) {
        let children: Vec<Tenant> = all_tenants
            .iter()
            .filter(|t| {
                t.parent_id
                    .as_ref()
                    .map(|p| p == &node.tenant.id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        for child_tenant in children {
            let mut child_node = TenantNode::new(child_tenant);
            Self::add_children(&mut child_node, all_tenants);
            node.add_child(child_node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wami::tenant::{Tenant, TenantId, TenantStatus, TenantType};

    fn create_test_tenant(id: TenantId, parent_id: Option<TenantId>) -> Tenant {
        Tenant {
            id: id.clone(),
            name: format!("tenant-{}", id),
            parent_id,
            organization: Some(format!("{} Organization", id.as_str())),
            status: TenantStatus::Active,
            tenant_type: TenantType::Root,
            provider_accounts: std::collections::HashMap::new(),
            arn: format!("arn:wami:tenant::{}", id.as_str()),
            providers: Vec::new(),
            created_at: chrono::Utc::now(),
            quotas: crate::wami::tenant::TenantQuotas::default(),
            quota_mode: crate::wami::tenant::QuotaMode::Inherited,
            max_child_depth: 5,
            can_create_sub_tenants: true,
            admin_principals: Vec::new(),
            metadata: std::collections::HashMap::new(),
            billing_info: None,
        }
    }

    #[test]
    fn test_tenant_node_new() {
        let tenant = create_test_tenant(TenantId::root(), None);
        let node = TenantNode::new(tenant.clone());
        assert_eq!(node.tenant.id.depth(), 0); // Root tenant
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_tenant_node_add_child() {
        let root_tenant = create_test_tenant(TenantId::root(), None);
        let mut root_node = TenantNode::new(root_tenant);

        let child_tenant = create_test_tenant(TenantId::root().child(), Some(TenantId::root()));
        let child_node = TenantNode::new(child_tenant);
        root_node.add_child(child_node);

        assert_eq!(root_node.children.len(), 1);
        assert_eq!(root_node.children[0].tenant.id.depth(), 1); // Child tenant
    }

    #[test]
    fn test_tenant_node_all_descendants() {
        let root_tenant = create_test_tenant(TenantId::root(), None);
        let mut root_node = TenantNode::new(root_tenant);

        let child1 = create_test_tenant(TenantId::root().child(), Some(TenantId::root()));
        let mut child1_node = TenantNode::new(child1);

        let grandchild = create_test_tenant(
            TenantId::root().child().child(),
            Some(TenantId::root().child()),
        );
        let grandchild_node = TenantNode::new(grandchild);
        child1_node.add_child(grandchild_node);

        let child2 = create_test_tenant(TenantId::root().child(), Some(TenantId::root()));
        let child2_node = TenantNode::new(child2);

        root_node.add_child(child1_node);
        root_node.add_child(child2_node);

        let descendants = root_node.all_descendants();
        assert_eq!(descendants.len(), 3);
        // Tenant IDs are now numeric, so we check by depth instead of string matching
        assert!(descendants.iter().any(|id| id.depth() == 1)); // child1 or child2
        assert!(descendants.iter().any(|id| id.depth() == 2)); // grandchild
        assert_eq!(descendants.iter().filter(|id| id.depth() == 1).count(), 2); // Two children
    }

    #[test]
    fn test_tenant_node_descendant_count() {
        let root_tenant = create_test_tenant(TenantId::root(), None);
        let mut root_node = TenantNode::new(root_tenant);

        let child1 = create_test_tenant(TenantId::root().child(), Some(TenantId::root()));
        let mut child1_node = TenantNode::new(child1);

        let grandchild = create_test_tenant(
            TenantId::root().child().child(),
            Some(TenantId::root().child()),
        );
        let grandchild_node = TenantNode::new(grandchild);
        child1_node.add_child(grandchild_node);

        let child2 = create_test_tenant(TenantId::root().child(), Some(TenantId::root()));
        let child2_node = TenantNode::new(child2);

        root_node.add_child(child1_node);
        root_node.add_child(child2_node);

        assert_eq!(root_node.descendant_count(), 3);
    }

    #[test]
    fn test_build_tree() {
        let root_id = TenantId::root();
        let root = create_test_tenant(root_id.clone(), None);
        let child1_id = root_id.child();
        let child1 = create_test_tenant(child1_id.clone(), Some(root_id.clone()));
        let child2_id = root_id.child();
        let child2 = create_test_tenant(child2_id.clone(), Some(root_id.clone()));
        let grandchild_id = child1_id.child();
        let grandchild = create_test_tenant(grandchild_id, Some(child1_id));

        let tenants = vec![
            root.clone(),
            child1.clone(),
            child2.clone(),
            grandchild.clone(),
        ];
        let tree = TenantNode::build_tree(tenants, &root_id); // Use the actual root_id

        assert!(tree.is_some());
        let root_node = tree.unwrap();
        assert_eq!(root_node.tenant.id.depth(), 0); // Root tenant
        assert_eq!(root_node.children.len(), 2);
        assert_eq!(root_node.descendant_count(), 3);
    }

    #[test]
    fn test_build_tree_no_root() {
        let child1 = create_test_tenant(TenantId::root().child(), Some(TenantId::root()));
        let tenants = vec![child1];
        let tree = TenantNode::build_tree(tenants, &TenantId::root());
        assert!(tree.is_none());
    }
}
