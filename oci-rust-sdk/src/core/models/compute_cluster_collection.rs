use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute clusters that match filter criteria, if any. A [compute cluster](https://docs.oracle.com/iaas/Content/Compute/Tasks/compute-clusters.htm) is a remote direct memory access (RDMA) network group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeClusterCollection {
    /// The list of compute clusters.
    pub items: Vec<ComputeClusterSummary>,
}

/// Required fields for ComputeClusterCollection
pub struct ComputeClusterCollectionRequired {
    /// The list of compute clusters.
    pub items: Vec<ComputeClusterSummary>,
}

impl ComputeClusterCollection {
    /// Create a new ComputeClusterCollection with required fields
    pub fn new(required: ComputeClusterCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeClusterSummary>) -> Self {
        self.items = value;
        self
    }
}
