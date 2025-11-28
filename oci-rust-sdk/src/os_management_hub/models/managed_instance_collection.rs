use serde::{Deserialize, Serialize};
use super::managed_instance_summary::ManagedInstanceSummary;

/// Collection of managed instances returned by list operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedInstanceCollection {
    /// List of managed instances
    pub items: Vec<ManagedInstanceSummary>,
}
