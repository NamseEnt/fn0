use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An array of CrossConnectMappingDetails
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossConnectMappingDetailsCollection {
    /// CrossConnectMappingDetails items
    pub items: Vec<CrossConnectMappingDetails>,
}

/// Required fields for CrossConnectMappingDetailsCollection
pub struct CrossConnectMappingDetailsCollectionRequired {
    /// CrossConnectMappingDetails items
    pub items: Vec<CrossConnectMappingDetails>,
}

impl CrossConnectMappingDetailsCollection {
    /// Create a new CrossConnectMappingDetailsCollection with required fields
    pub fn new(required: CrossConnectMappingDetailsCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<CrossConnectMappingDetails>) -> Self {
        self.items = value;
        self
    }
}
