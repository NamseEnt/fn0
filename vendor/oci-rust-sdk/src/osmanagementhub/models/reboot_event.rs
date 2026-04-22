use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides information for a reboot event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebootEvent {
    pub data: RebootEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for RebootEvent
pub struct RebootEventRequired {
    pub data: RebootEventData,

    pub r#type: String,
}

impl RebootEvent {
    /// Create a new RebootEvent with required fields
    pub fn new(required: RebootEventRequired) -> Self {
        Self {
            data: required.data,

            r#type: required.r#type,
        }
    }

    /// Set data
    pub fn set_data(mut self, value: RebootEventData) -> Self {
        self.data = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
