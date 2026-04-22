use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of module streams returned for {@link #listModuleStreamProfiles(ListModuleStreamProfilesRequest) listModuleStreamProfiles} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStreamCollection {
    /// List of module streams.
    pub items: Vec<ModuleStreamSummary>,
}

/// Required fields for ModuleStreamCollection
pub struct ModuleStreamCollectionRequired {
    /// List of module streams.
    pub items: Vec<ModuleStreamSummary>,
}

impl ModuleStreamCollection {
    /// Create a new ModuleStreamCollection with required fields
    pub fn new(required: ModuleStreamCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ModuleStreamSummary>) -> Self {
        self.items = value;
        self
    }
}
