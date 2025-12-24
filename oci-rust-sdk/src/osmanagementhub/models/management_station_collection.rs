use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of management stations returned for the {@link #listManagementStations(ListManagementStationsRequest) listManagementStations} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementStationCollection {
    /// List of management stations.
    pub items: Vec<ManagementStationSummary>,
}

/// Required fields for ManagementStationCollection
pub struct ManagementStationCollectionRequired {
    /// List of management stations.
    pub items: Vec<ManagementStationSummary>,
}

impl ManagementStationCollection {
    /// Create a new ManagementStationCollection with required fields
    pub fn new(required: ManagementStationCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagementStationSummary>) -> Self {
        self.items = value;
        self
    }
}
