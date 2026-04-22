use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute GPU memory clusters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGpuMemoryClusterCollection {
    /// The list of compute GPU memory clusters.
    pub items: Vec<ComputeGpuMemoryClusterSummary>,
}

/// Required fields for ComputeGpuMemoryClusterCollection
pub struct ComputeGpuMemoryClusterCollectionRequired {
    /// The list of compute GPU memory clusters.
    pub items: Vec<ComputeGpuMemoryClusterSummary>,
}

impl ComputeGpuMemoryClusterCollection {
    /// Create a new ComputeGpuMemoryClusterCollection with required fields
    pub fn new(required: ComputeGpuMemoryClusterCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeGpuMemoryClusterSummary>) -> Self {
        self.items = value;
        self
    }
}
