use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute capacity topologies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeCapacityTopologyCollection {
    /// The list of compute capacity topologies.
    pub items: Vec<ComputeCapacityTopologySummary>,
}

/// Required fields for ComputeCapacityTopologyCollection
pub struct ComputeCapacityTopologyCollectionRequired {
    /// The list of compute capacity topologies.
    pub items: Vec<ComputeCapacityTopologySummary>,
}

impl ComputeCapacityTopologyCollection {
    /// Create a new ComputeCapacityTopologyCollection with required fields
    pub fn new(required: ComputeCapacityTopologyCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeCapacityTopologySummary>) -> Self {
        self.items = value;
        self
    }
}
