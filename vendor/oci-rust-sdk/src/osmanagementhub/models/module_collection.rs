use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of modules returned for the {@link #searchSoftwareSourceModules(SearchSoftwareSourceModulesRequest) searchSoftwareSourceModules} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleCollection {
    /// List of modules.
    pub items: Vec<ModuleSummary>,
}

/// Required fields for ModuleCollection
pub struct ModuleCollectionRequired {
    /// List of modules.
    pub items: Vec<ModuleSummary>,
}

impl ModuleCollection {
    /// Create a new ModuleCollection with required fields
    pub fn new(required: ModuleCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ModuleSummary>) -> Self {
        self.items = value;
        self
    }
}
