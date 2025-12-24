use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of lifecycle environments returned for the {@link #listLifecycleEnvironments(ListLifecycleEnvironmentsRequest) listLifecycleEnvironments} operation. The list contains a summary of each lifecycle environment and other information, such as metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleEnvironmentCollection {
    /// List of lifecycle environments.
    pub items: Vec<LifecycleEnvironmentSummary>,
}

/// Required fields for LifecycleEnvironmentCollection
pub struct LifecycleEnvironmentCollectionRequired {
    /// List of lifecycle environments.
    pub items: Vec<LifecycleEnvironmentSummary>,
}

impl LifecycleEnvironmentCollection {
    /// Create a new LifecycleEnvironmentCollection with required fields
    pub fn new(required: LifecycleEnvironmentCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<LifecycleEnvironmentSummary>) -> Self {
        self.items = value;
        self
    }
}
