use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides information about the management station event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementStationEvent {
    pub data: ManagementStationEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for ManagementStationEvent
pub struct ManagementStationEventRequired {
    pub data: ManagementStationEventData,

    pub r#type: String,
}

impl ManagementStationEvent {
    /// Create a new ManagementStationEvent with required fields
    pub fn new(required: ManagementStationEventRequired) -> Self {
        Self {
            data: required.data,

            r#type: required.r#type,
        }
    }

    /// Set data
    pub fn set_data(mut self, value: ManagementStationEventData) -> Self {
        self.data = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
