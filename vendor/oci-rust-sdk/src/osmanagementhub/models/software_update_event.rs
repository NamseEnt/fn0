use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides information for a software update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareUpdateEvent {
    pub data: SoftwareUpdateEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for SoftwareUpdateEvent
pub struct SoftwareUpdateEventRequired {
    pub data: SoftwareUpdateEventData,

    pub r#type: String,
}

impl SoftwareUpdateEvent {
    /// Create a new SoftwareUpdateEvent with required fields
    pub fn new(required: SoftwareUpdateEventRequired) -> Self {
        Self {
            data: required.data,

            r#type: required.r#type,
        }
    }

    /// Set data
    pub fn set_data(mut self, value: SoftwareUpdateEventData) -> Self {
        self.data = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
