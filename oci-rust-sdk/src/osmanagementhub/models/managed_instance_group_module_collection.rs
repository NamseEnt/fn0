use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of module streams returned for the {@link #listManagedInstanceGroupModules(ListManagedInstanceGroupModulesRequest) listManagedInstanceGroupModules} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupModuleCollection {
    /// List of module streams.
    pub items: Vec<ManagedInstanceGroupModuleSummary>,
}

/// Required fields for ManagedInstanceGroupModuleCollection
pub struct ManagedInstanceGroupModuleCollectionRequired {
    /// List of module streams.
    pub items: Vec<ManagedInstanceGroupModuleSummary>,
}

impl ManagedInstanceGroupModuleCollection {
    /// Create a new ManagedInstanceGroupModuleCollection with required fields
    pub fn new(required: ManagedInstanceGroupModuleCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceGroupModuleSummary>) -> Self {
        self.items = value;
        self
    }
}
