use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides information for a software source event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSourceEvent {
    pub data: SoftwareSourceEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for SoftwareSourceEvent
pub struct SoftwareSourceEventRequired {
    pub data: SoftwareSourceEventData,

    pub r#type: String,
}

impl SoftwareSourceEvent {
    /// Create a new SoftwareSourceEvent with required fields
    pub fn new(required: SoftwareSourceEventRequired) -> Self {
        Self {
            data: required.data,

            r#type: required.r#type,
        }
    }

    /// Set data
    pub fn set_data(mut self, value: SoftwareSourceEventData) -> Self {
        self.data = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
