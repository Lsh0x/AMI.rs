//! Permissions Boundary Request and Response Types

use serde::{Deserialize, Serialize};

/// Type of principal for permissions boundary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrincipalType {
    /// IAM User
    User,
    /// IAM Role
    Role,
}

/// Request to attach a permissions boundary to a user or role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutPermissionsBoundaryRequest {
    /// Type of principal (User or Role)
    pub principal_type: PrincipalType,
    /// Name of the user or role
    pub principal_name: String,
    /// ARN of the policy to use as the permissions boundary
    pub permissions_boundary: String,
}

/// Request to remove a permissions boundary from a user or role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePermissionsBoundaryRequest {
    /// Type of principal (User or Role)
    pub principal_type: PrincipalType,
    /// Name of the user or role
    pub principal_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_request_creation() {
        let request = PutPermissionsBoundaryRequest {
            principal_type: PrincipalType::User,
            principal_name: "alice".to_string(),
            permissions_boundary: "arn:aws:iam::123456789012:policy/boundary".to_string(),
        };

        assert_eq!(request.principal_type, PrincipalType::User);
        assert_eq!(request.principal_name, "alice");
    }

    #[test]
    fn test_delete_request_creation() {
        let request = DeletePermissionsBoundaryRequest {
            principal_type: PrincipalType::Role,
            principal_name: "admin-role".to_string(),
        };

        assert_eq!(request.principal_type, PrincipalType::Role);
        assert_eq!(request.principal_name, "admin-role");
    }

    #[test]
    fn test_principal_type_equality() {
        assert_eq!(PrincipalType::User, PrincipalType::User);
        assert_eq!(PrincipalType::Role, PrincipalType::Role);
        assert_ne!(PrincipalType::User, PrincipalType::Role);
    }
}
