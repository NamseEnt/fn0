use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of work requests returned for the {@link #listWorkRequests(ListWorkRequestsRequest) listWorkRequests} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestSummaryCollection {
    /// List of work requests.
    pub items: Vec<WorkRequestSummary>,
}

/// Required fields for WorkRequestSummaryCollection
pub struct WorkRequestSummaryCollectionRequired {
    /// List of work requests.
    pub items: Vec<WorkRequestSummary>,
}

impl WorkRequestSummaryCollection {
    /// Create a new WorkRequestSummaryCollection with required fields
    pub fn new(required: WorkRequestSummaryCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<WorkRequestSummary>) -> Self {
        self.items = value;
        self
    }
}
