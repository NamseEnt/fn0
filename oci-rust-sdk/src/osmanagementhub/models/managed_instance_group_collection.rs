use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of managed instance groups returned for the {@link #listManagedInstanceGroups(ListManagedInstanceGroupsRequest) listManagedInstanceGroups} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupCollection {
    /// List of managed instance groups.
    pub items: Vec<ManagedInstanceGroupSummary>,
}

/// Required fields for ManagedInstanceGroupCollection
pub struct ManagedInstanceGroupCollectionRequired {
    /// List of managed instance groups.
    pub items: Vec<ManagedInstanceGroupSummary>,
}

impl ManagedInstanceGroupCollection {
    /// Create a new ManagedInstanceGroupCollection with required fields
    pub fn new(required: ManagedInstanceGroupCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceGroupSummary>) -> Self {
        self.items = value;
        self
    }
}
