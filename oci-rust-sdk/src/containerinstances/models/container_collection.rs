use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of containers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerCollection {
    /// List of containers.
    pub items: Vec<ContainerSummary>,
}

/// Required fields for ContainerCollection
pub struct ContainerCollectionRequired {
    /// List of containers.
    pub items: Vec<ContainerSummary>,
}

impl ContainerCollection {
    /// Create a new ContainerCollection with required fields
    pub fn new(required: ContainerCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ContainerSummary>) -> Self {
        self.items = value;
        self
    }
}
