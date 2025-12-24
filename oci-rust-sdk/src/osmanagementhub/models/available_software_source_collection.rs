use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of available software sources returned for the {@link #listManagedInstanceAvailableSoftwareSources(ListManagedInstanceAvailableSoftwareSourcesRequest) listManagedInstanceAvailableSoftwareSources} or the {@link #listManagedInstanceGroupAvailableSoftwareSources(ListManagedInstanceGroupAvailableSoftwareSourcesRequest) listManagedInstanceGroupAvailableSoftwareSources} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableSoftwareSourceCollection {
    /// List of available software sources.
    pub items: Vec<AvailableSoftwareSourceSummary>,
}

/// Required fields for AvailableSoftwareSourceCollection
pub struct AvailableSoftwareSourceCollectionRequired {
    /// List of available software sources.
    pub items: Vec<AvailableSoftwareSourceSummary>,
}

impl AvailableSoftwareSourceCollection {
    /// Create a new AvailableSoftwareSourceCollection with required fields
    pub fn new(required: AvailableSoftwareSourceCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<AvailableSoftwareSourceSummary>) -> Self {
        self.items = value;
        self
    }
}
