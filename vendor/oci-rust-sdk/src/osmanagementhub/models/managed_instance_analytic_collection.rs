use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of managed instance metrics returned for the {@link #summarizeManagedInstanceAnalytics(SummarizeManagedInstanceAnalyticsRequest) summarizeManagedInstanceAnalytics} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceAnalyticCollection {
    /// List of managed instance analytic summary objects.
    pub items: Vec<ManagedInstanceAnalyticSummary>,
}

/// Required fields for ManagedInstanceAnalyticCollection
pub struct ManagedInstanceAnalyticCollectionRequired {
    /// List of managed instance analytic summary objects.
    pub items: Vec<ManagedInstanceAnalyticSummary>,
}

impl ManagedInstanceAnalyticCollection {
    /// Create a new ManagedInstanceAnalyticCollection with required fields
    pub fn new(required: ManagedInstanceAnalyticCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceAnalyticSummary>) -> Self {
        self.items = value;
        self
    }
}
