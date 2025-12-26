use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute GPU memory fabrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGpuMemoryFabricCollection {
    /// The list of compute GPU memory fabrics.
    pub items: Vec<ComputeGpuMemoryFabricSummary>,
}

/// Required fields for ComputeGpuMemoryFabricCollection
pub struct ComputeGpuMemoryFabricCollectionRequired {
    /// The list of compute GPU memory fabrics.
    pub items: Vec<ComputeGpuMemoryFabricSummary>,
}

impl ComputeGpuMemoryFabricCollection {
    /// Create a new ComputeGpuMemoryFabricCollection with required fields
    pub fn new(required: ComputeGpuMemoryFabricCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeGpuMemoryFabricSummary>) -> Self {
        self.items = value;
        self
    }
}
