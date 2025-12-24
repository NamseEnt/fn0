use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Results of a workRequestLog search. Contains both workRequestLog items and other information, such as metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestLogEntryCollection {
    /// List of workRequestLogEntries.
    pub items: Vec<WorkRequestLogEntry>,
}

/// Required fields for WorkRequestLogEntryCollection
pub struct WorkRequestLogEntryCollectionRequired {
    /// List of workRequestLogEntries.
    pub items: Vec<WorkRequestLogEntry>,
}

impl WorkRequestLogEntryCollection {
    /// Create a new WorkRequestLogEntryCollection with required fields
    pub fn new(required: WorkRequestLogEntryCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<WorkRequestLogEntry>) -> Self {
        self.items = value;
        self
    }
}
