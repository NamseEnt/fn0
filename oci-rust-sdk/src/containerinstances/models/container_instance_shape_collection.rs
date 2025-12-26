use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A collection of container instance shapes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInstanceShapeCollection {
    /// A list of shapes.
    pub items: Vec<ContainerInstanceShapeSummary>,
}

/// Required fields for ContainerInstanceShapeCollection
pub struct ContainerInstanceShapeCollectionRequired {
    /// A list of shapes.
    pub items: Vec<ContainerInstanceShapeSummary>,
}

impl ContainerInstanceShapeCollection {
    /// Create a new ContainerInstanceShapeCollection with required fields
    pub fn new(required: ContainerInstanceShapeCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ContainerInstanceShapeSummary>) -> Self {
        self.items = value;
        self
    }
}
