use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The results returned by a {@code ListByoasn} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoasnCollection {
    /// A list of {@code Byoasn} resource summaries.
    pub items: Vec<ByoasnSummary>,
}

/// Required fields for ByoasnCollection
pub struct ByoasnCollectionRequired {
    /// A list of {@code Byoasn} resource summaries.
    pub items: Vec<ByoasnSummary>,
}

impl ByoasnCollection {
    /// Create a new ByoasnCollection with required fields
    pub fn new(required: ByoasnCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ByoasnSummary>) -> Self {
        self.items = value;
        self
    }
}
