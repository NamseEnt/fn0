use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of lifecycle stages returned for the {@link #listLifecycleStages(ListLifecycleStagesRequest) listLifecycleStages} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleStageCollection {
    /// List of lifecycle stages.
    pub items: Vec<LifecycleStageSummary>,
}

/// Required fields for LifecycleStageCollection
pub struct LifecycleStageCollectionRequired {
    /// List of lifecycle stages.
    pub items: Vec<LifecycleStageSummary>,
}

impl LifecycleStageCollection {
    /// Create a new LifecycleStageCollection with required fields
    pub fn new(required: LifecycleStageCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<LifecycleStageSummary>) -> Self {
        self.items = value;
        self
    }
}
