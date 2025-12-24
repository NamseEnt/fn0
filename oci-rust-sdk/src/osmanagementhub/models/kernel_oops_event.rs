use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the kernel OOPS event. A kernel OOPS event occurs when the kernel detects an erroneous state inside itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelOopsEvent {
    pub data: KernelEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for KernelOopsEvent
pub struct KernelOopsEventRequired {
    pub data: KernelEventData,

    pub r#type: String,
}

impl KernelOopsEvent {
    /// Create a new KernelOopsEvent with required fields
    pub fn new(required: KernelOopsEventRequired) -> Self {
        Self {
            data: required.data,

            r#type: required.r#type,
        }
    }

    /// Set data
    pub fn set_data(mut self, value: KernelEventData) -> Self {
        self.data = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
