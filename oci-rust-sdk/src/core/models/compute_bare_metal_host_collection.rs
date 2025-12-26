use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of compute bare metal hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeBareMetalHostCollection {
    /// The list of compute bare metal hosts.
    pub items: Vec<ComputeBareMetalHostSummary>,
}

/// Required fields for ComputeBareMetalHostCollection
pub struct ComputeBareMetalHostCollectionRequired {
    /// The list of compute bare metal hosts.
    pub items: Vec<ComputeBareMetalHostSummary>,
}

impl ComputeBareMetalHostCollection {
    /// Create a new ComputeBareMetalHostCollection with required fields
    pub fn new(required: ComputeBareMetalHostCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ComputeBareMetalHostSummary>) -> Self {
        self.items = value;
        self
    }
}
