use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute GPU memory cluster instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGpuMemoryClusterInstanceCollection {
    /// The list of compute GPU memory cluster instances.
    pub items: Vec<ComputeGpuMemoryClusterInstanceSummary>,
}

/// Required fields for ComputeGpuMemoryClusterInstanceCollection
pub struct ComputeGpuMemoryClusterInstanceCollectionRequired {
    /// The list of compute GPU memory cluster instances.
    pub items: Vec<ComputeGpuMemoryClusterInstanceSummary>,
}

impl ComputeGpuMemoryClusterInstanceCollection {
    /// Create a new ComputeGpuMemoryClusterInstanceCollection with required fields
    pub fn new(required: ComputeGpuMemoryClusterInstanceCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeGpuMemoryClusterInstanceSummary>) -> Self {
        self.items = value;
        self
    }
}
