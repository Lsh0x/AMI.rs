//! Policies Service Module
//!
//! Services for managing IAM policies and policy evaluation.

pub mod attachment;
pub mod evaluation;
pub mod inline;
pub mod permissions_boundary;
pub mod policy;

pub use attachment::AttachmentService;
pub use evaluation::EvaluationService;
pub use inline::InlinePolicyService;
pub use permissions_boundary::PermissionsBoundaryService;
pub use policy::PolicyService;
