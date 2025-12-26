use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The results returned by a {@code ListByoipRange} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoipRangeCollection {
    /// A list of {@code ByoipRange} resource summaries.
    pub items: Vec<ByoipRangeSummary>,
}

/// Required fields for ByoipRangeCollection
pub struct ByoipRangeCollectionRequired {
    /// A list of {@code ByoipRange} resource summaries.
    pub items: Vec<ByoipRangeSummary>,
}

impl ByoipRangeCollection {
    /// Create a new ByoipRangeCollection with required fields
    pub fn new(required: ByoipRangeCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ByoipRangeSummary>) -> Self {
        self.items = value;
        self
    }
}
