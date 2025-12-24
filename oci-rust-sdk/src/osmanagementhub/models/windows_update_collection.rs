use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of Windows updates returned for the {@link #listWindowsUpdates(ListWindowsUpdatesRequest) listWindowsUpdates} operation. The list contains a summary of each update.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsUpdateCollection {
    /// List of Windows updates.
    pub items: Vec<WindowsUpdateSummary>,
}

/// Required fields for WindowsUpdateCollection
pub struct WindowsUpdateCollectionRequired {
    /// List of Windows updates.
    pub items: Vec<WindowsUpdateSummary>,
}

impl WindowsUpdateCollection {
    /// Create a new WindowsUpdateCollection with required fields
    pub fn new(required: WindowsUpdateCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<WindowsUpdateSummary>) -> Self {
        self.items = value;
        self
    }
}
