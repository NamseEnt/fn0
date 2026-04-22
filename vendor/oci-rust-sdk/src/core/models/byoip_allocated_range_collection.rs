use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Results of a {@code ListByoipAllocatedRanges} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoipAllocatedRangeCollection {
    /// A list of subranges of a BYOIP CIDR block allocated to an IP pool.
    pub items: Vec<ByoipAllocatedRangeSummary>,
}

/// Required fields for ByoipAllocatedRangeCollection
pub struct ByoipAllocatedRangeCollectionRequired {
    /// A list of subranges of a BYOIP CIDR block allocated to an IP pool.
    pub items: Vec<ByoipAllocatedRangeSummary>,
}

impl ByoipAllocatedRangeCollection {
    /// Create a new ByoipAllocatedRangeCollection with required fields
    pub fn new(required: ByoipAllocatedRangeCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ByoipAllocatedRangeSummary>) -> Self {
        self.items = value;
        self
    }
}
