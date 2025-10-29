//! Policy simulation and evaluation

pub mod model;
// pub mod operations; // TODO: Fix model ref
pub mod requests;

// Re-export types
pub use model::{ContextEntry, EvaluationResult, StatementMatch};
pub use requests::{
    SimulateCustomPolicyRequest, SimulatePolicyResponse, SimulatePrincipalPolicyRequest,
};
