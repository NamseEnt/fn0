use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides information for a Ksplice update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KspliceUpdateEvent {
    pub data: KspliceUpdateEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for KspliceUpdateEvent
pub struct KspliceUpdateEventRequired {
    pub data: KspliceUpdateEventData,

    pub r#type: String,
}

impl KspliceUpdateEvent {
    /// Create a new KspliceUpdateEvent with required fields
    pub fn new(required: KspliceUpdateEventRequired) -> Self {
        Self {
            data: required.data,

            r#type: required.r#type,
        }
    }

    /// Set data
    pub fn set_data(mut self, value: KspliceUpdateEventData) -> Self {
        self.data = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
