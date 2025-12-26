use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Summary information about a list of container instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInstanceCollection {
    /// List of container instances.
    pub items: Vec<ContainerInstanceSummary>,
}

/// Required fields for ContainerInstanceCollection
pub struct ContainerInstanceCollectionRequired {
    /// List of container instances.
    pub items: Vec<ContainerInstanceSummary>,
}

impl ContainerInstanceCollection {
    /// Create a new ContainerInstanceCollection with required fields
    pub fn new(required: ContainerInstanceCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ContainerInstanceSummary>) -> Self {
        self.items = value;
        self
    }
}
