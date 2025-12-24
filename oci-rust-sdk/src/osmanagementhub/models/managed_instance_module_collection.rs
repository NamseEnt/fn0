use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of modules returned for the {@link #listManagedInstanceModules(ListManagedInstanceModulesRequest) listManagedInstanceModules} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceModuleCollection {
    /// List of module streams.
    pub items: Vec<ManagedInstanceModuleSummary>,
}

/// Required fields for ManagedInstanceModuleCollection
pub struct ManagedInstanceModuleCollectionRequired {
    /// List of module streams.
    pub items: Vec<ManagedInstanceModuleSummary>,
}

impl ManagedInstanceModuleCollection {
    /// Create a new ManagedInstanceModuleCollection with required fields
    pub fn new(required: ManagedInstanceModuleCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceModuleSummary>) -> Self {
        self.items = value;
        self
    }
}
