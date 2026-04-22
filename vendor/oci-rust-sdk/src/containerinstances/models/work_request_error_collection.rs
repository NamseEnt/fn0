use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Results of a workRequestError search. Contains both WorkRequestError items and other information, such as metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestErrorCollection {
    /// List of workRequestError objects.
    pub items: Vec<WorkRequestError>,
}

/// Required fields for WorkRequestErrorCollection
pub struct WorkRequestErrorCollectionRequired {
    /// List of workRequestError objects.
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
