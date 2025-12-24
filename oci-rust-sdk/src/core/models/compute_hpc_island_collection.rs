use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute HPC islands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeHpcIslandCollection {
    /// The list of compute HPC islands.
    pub items: Vec<ComputeHpcIslandSummary>,
}

/// Required fields for ComputeHpcIslandCollection
pub struct ComputeHpcIslandCollectionRequired {
    /// The list of compute HPC islands.
    pub items: Vec<ComputeHpcIslandSummary>,
}

impl ComputeHpcIslandCollection {
    /// Create a new ComputeHpcIslandCollection with required fields
    pub fn new(required: ComputeHpcIslandCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeHpcIslandSummary>) -> Self {
        self.items = value;
        self
    }
}
