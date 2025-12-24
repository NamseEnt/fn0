use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of available Windows updates returned for the {@link #listManagedInstanceAvailableWindowsUpdates(ListManagedInstanceAvailableWindowsUpdatesRequest) listManagedInstanceAvailableWindowsUpdates} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableWindowsUpdateCollection {
    /// List of available Windows updates.
    pub items: Vec<AvailableWindowsUpdateSummary>,
}

/// Required fields for AvailableWindowsUpdateCollection
pub struct AvailableWindowsUpdateCollectionRequired {
    /// List of available Windows updates.
    pub items: Vec<AvailableWindowsUpdateSummary>,
}

impl AvailableWindowsUpdateCollection {
    /// Create a new AvailableWindowsUpdateCollection with required fields
    pub fn new(required: AvailableWindowsUpdateCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<AvailableWindowsUpdateSummary>) -> Self {
        self.items = value;
        self
    }
}
