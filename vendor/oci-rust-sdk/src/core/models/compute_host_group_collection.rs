use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute host groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeHostGroupCollection {
    /// The list of compute host groups.
    pub items: Vec<ComputeHostGroupSummary>,
}

/// Required fields for ComputeHostGroupCollection
pub struct ComputeHostGroupCollectionRequired {
    /// The list of compute host groups.
    pub items: Vec<ComputeHostGroupSummary>,
}

impl ComputeHostGroupCollection {
    /// Create a new ComputeHostGroupCollection with required fields
    pub fn new(required: ComputeHostGroupCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeHostGroupSummary>) -> Self {
        self.items = value;
        self
    }
}
