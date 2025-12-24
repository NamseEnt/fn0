use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of errata returned for the {@link #listManagedInstanceErrata(ListManagedInstanceErrataRequest) listManagedInstanceErrata} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceErratumSummaryCollection {
    /// List of errata.
    pub items: Vec<ManagedInstanceErratumSummary>,
}

/// Required fields for ManagedInstanceErratumSummaryCollection
pub struct ManagedInstanceErratumSummaryCollectionRequired {
    /// List of errata.
    pub items: Vec<ManagedInstanceErratumSummary>,
}

impl ManagedInstanceErratumSummaryCollection {
    /// Create a new ManagedInstanceErratumSummaryCollection with required fields
    pub fn new(required: ManagedInstanceErratumSummaryCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceErratumSummary>) -> Self {
        self.items = value;
        self
    }
}
