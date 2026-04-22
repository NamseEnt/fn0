use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A summary representation of resources that matched the search criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSummaryCollection {
    /// A list of resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<ResourceSummary>>,
}

impl ResourceSummaryCollection {
    /// Create a new ResourceSummaryCollection
    pub fn new() -> Self {
        Self { items: None }
    }

    /// Set items
    pub fn set_items(mut self, value: Option<Vec<ResourceSummary>>) -> Self {
        self.items = value;
        self
    }

    /// Set items (unwraps Option)
    pub fn with_items(mut self, value: Vec<ResourceSummary>) -> Self {
        self.items = Some(value);
        self
    }
}

impl Default for ResourceSummaryCollection {
    fn default() -> Self {
        Self::new()
    }
}
