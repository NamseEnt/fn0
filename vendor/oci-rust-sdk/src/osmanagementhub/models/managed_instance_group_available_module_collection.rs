use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of modules returned for the {@link #listManagedInstanceGroupAvailableModules(ListManagedInstanceGroupAvailableModulesRequest) listManagedInstanceGroupAvailableModules} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupAvailableModuleCollection {
    /// List of available modules.
    pub items: Vec<ManagedInstanceGroupAvailableModuleSummary>,
}

/// Required fields for ManagedInstanceGroupAvailableModuleCollection
pub struct ManagedInstanceGroupAvailableModuleCollectionRequired {
    /// List of available modules.
    pub items: Vec<ManagedInstanceGroupAvailableModuleSummary>,
}

impl ManagedInstanceGroupAvailableModuleCollection {
    /// Create a new ManagedInstanceGroupAvailableModuleCollection with required fields
    pub fn new(required: ManagedInstanceGroupAvailableModuleCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceGroupAvailableModuleSummary>) -> Self {
        self.items = value;
        self
    }
}
