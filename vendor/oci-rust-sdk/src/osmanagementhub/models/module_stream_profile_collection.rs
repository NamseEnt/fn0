use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of module stream profiles returned for the {@link #listModuleStreamProfiles(ListModuleStreamProfilesRequest) listModuleStreamProfiles} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStreamProfileCollection {
    /// List of module stream profiles.
    pub items: Vec<ModuleStreamProfileSummary>,
}

/// Required fields for ModuleStreamProfileCollection
pub struct ModuleStreamProfileCollectionRequired {
    /// List of module stream profiles.
    pub items: Vec<ModuleStreamProfileSummary>,
}

impl ModuleStreamProfileCollection {
    /// Create a new ModuleStreamProfileCollection with required fields
    pub fn new(required: ModuleStreamProfileCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ModuleStreamProfileSummary>) -> Self {
        self.items = value;
        self
    }
}
