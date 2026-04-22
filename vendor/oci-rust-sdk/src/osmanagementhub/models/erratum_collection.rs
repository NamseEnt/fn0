use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of errata returned for the {@link #listErrata(ListErrataRequest) listErrata} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErratumCollection {
    /// List of errata.
    pub items: Vec<ErratumSummary>,
}

/// Required fields for ErratumCollection
pub struct ErratumCollectionRequired {
    /// List of errata.
    pub items: Vec<ErratumSummary>,
}

impl ErratumCollection {
    /// Create a new ErratumCollection with required fields
    pub fn new(required: ErratumCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ErratumSummary>) -> Self {
        self.items = value;
        self
    }
}
