use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of software sources returned for the {@link #listSoftwareSources(ListSoftwareSourcesRequest) listSoftwareSources} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSourceCollection {
    /// List of software sources.
    pub items: Vec<SoftwareSourceSummary>,
}

/// Required fields for SoftwareSourceCollection
pub struct SoftwareSourceCollectionRequired {
    /// List of software sources.
    pub items: Vec<SoftwareSourceSummary>,
}

impl SoftwareSourceCollection {
    /// Create a new SoftwareSourceCollection with required fields
    pub fn new(required: SoftwareSourceCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<SoftwareSourceSummary>) -> Self {
        self.items = value;
        self
    }
}
