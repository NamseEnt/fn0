use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of work request errors returned for the {@link #listWorkRequestErrors(ListWorkRequestErrorsRequest) listWorkRequestErrors} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestErrorCollection {
    /// List of work request error objects.
    pub items: Vec<WorkRequestError>,
}

/// Required fields for WorkRequestErrorCollection
pub struct WorkRequestErrorCollectionRequired {
    /// List of work request error objects.
    pub items: Vec<WorkRequestError>,
}

impl WorkRequestErrorCollection {
    /// Create a new WorkRequestErrorCollection with required fields
    pub fn new(required: WorkRequestErrorCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<WorkRequestError>) -> Self {
        self.items = value;
        self
    }
}
