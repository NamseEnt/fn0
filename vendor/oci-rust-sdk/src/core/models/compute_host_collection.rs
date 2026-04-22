use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeHostCollection {
    /// The list of compute hosts.
    pub items: Vec<ComputeHostSummary>,
}

/// Required fields for ComputeHostCollection
pub struct ComputeHostCollectionRequired {
    /// The list of compute hosts.
    pub items: Vec<ComputeHostSummary>,
}

impl ComputeHostCollection {
    /// Create a new ComputeHostCollection with required fields
    pub fn new(required: ComputeHostCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeHostSummary>) -> Self {
        self.items = value;
        self
    }
}
