use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of installed Windows updates returned for the {@link #listManagedInstanceInstalledWindowsUpdates(ListManagedInstanceInstalledWindowsUpdatesRequest) listManagedInstanceInstalledWindowsUpdates} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledWindowsUpdateCollection {
    /// List of installed Windows updates.
    pub items: Vec<InstalledWindowsUpdateSummary>,
}

/// Required fields for InstalledWindowsUpdateCollection
pub struct InstalledWindowsUpdateCollectionRequired {
    /// List of installed Windows updates.
    pub items: Vec<InstalledWindowsUpdateSummary>,
}

impl InstalledWindowsUpdateCollection {
    /// Create a new InstalledWindowsUpdateCollection with required fields
    pub fn new(required: InstalledWindowsUpdateCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<InstalledWindowsUpdateSummary>) -> Self {
        self.items = value;
        self
    }
}
