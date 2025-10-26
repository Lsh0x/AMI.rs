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
