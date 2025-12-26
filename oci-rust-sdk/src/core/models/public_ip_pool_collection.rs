use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Results of a {@code ListPublicIpPool} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIpPoolCollection {
    /// A list of public IP pool summaries.
    pub items: Vec<PublicIpPoolSummary>,
}

/// Required fields for PublicIpPoolCollection
pub struct PublicIpPoolCollectionRequired {
    /// A list of public IP pool summaries.
    pub items: Vec<PublicIpPoolSummary>,
}

impl PublicIpPoolCollection {
    /// Create a new PublicIpPoolCollection with required fields
    pub fn new(required: PublicIpPoolCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<PublicIpPoolSummary>) -> Self {
        self.items = value;
        self
    }
}
