use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of scheduled jobs returned for the {@link #listScheduledJobs(ListScheduledJobsRequest) listScheduledJobs} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledJobCollection {
    /// List of scheduled jobs.
    pub items: Vec<ScheduledJobSummary>,
}

/// Required fields for ScheduledJobCollection
pub struct ScheduledJobCollectionRequired {
    /// List of scheduled jobs.
    pub items: Vec<ScheduledJobSummary>,
}

impl ScheduledJobCollection {
    /// Create a new ScheduledJobCollection with required fields
    pub fn new(required: ScheduledJobCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ScheduledJobSummary>) -> Self {
        self.items = value;
        self
    }
}
