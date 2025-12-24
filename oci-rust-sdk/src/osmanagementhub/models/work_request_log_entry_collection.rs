use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of work request logs returned for the {@link #listWorkRequestLogs(ListWorkRequestLogsRequest) listWorkRequestLogs} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestLogEntryCollection {
    /// List of work request log entries.
    pub items: Vec<WorkRequestLogEntry>,
}

/// Required fields for WorkRequestLogEntryCollection
pub struct WorkRequestLogEntryCollectionRequired {
    /// List of work request log entries.
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
