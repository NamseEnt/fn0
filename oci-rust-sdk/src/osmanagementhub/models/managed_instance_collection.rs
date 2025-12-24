use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of managed instances returned for the {@link #listManagedInstances(ListManagedInstancesRequest) listManagedInstances} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceCollection {
    /// List of managed instances.
    pub items: Vec<ManagedInstanceSummary>,
}

/// Required fields for ManagedInstanceCollection
pub struct ManagedInstanceCollectionRequired {
    /// List of managed instances.
    pub items: Vec<ManagedInstanceSummary>,
}

impl ManagedInstanceCollection {
    /// Create a new ManagedInstanceCollection with required fields
    pub fn new(required: ManagedInstanceCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceSummary>) -> Self {
        self.items = value;
        self
    }
}
