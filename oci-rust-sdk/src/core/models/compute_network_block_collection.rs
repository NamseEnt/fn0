use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute network blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeNetworkBlockCollection {
    /// The list of compute network blocks.
    pub items: Vec<ComputeNetworkBlockSummary>,
}

/// Required fields for ComputeNetworkBlockCollection
pub struct ComputeNetworkBlockCollectionRequired {
    /// The list of compute network blocks.
    pub items: Vec<ComputeNetworkBlockSummary>,
}

impl ComputeNetworkBlockCollection {
    /// Create a new ComputeNetworkBlockCollection with required fields
    pub fn new(required: ComputeNetworkBlockCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeNetworkBlockSummary>) -> Self {
        self.items = value;
        self
    }
}
