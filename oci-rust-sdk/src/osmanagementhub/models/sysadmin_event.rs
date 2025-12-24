use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the sysadmin event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysadminEvent {
    pub data: SysadminEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for SysadminEvent
pub struct SysadminEventRequired {
    pub data: SysadminEventData,

    pub r#type: String,
}

impl SysadminEvent {
    /// Create a new SysadminEvent with required fields
    pub fn new(required: SysadminEventRequired) -> Self {
        Self {
            data: required.data,

            r#type: required.r#type,
        }
    }

    /// Set data
    pub fn set_data(mut self, value: SysadminEventData) -> Self {
        self.data = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
