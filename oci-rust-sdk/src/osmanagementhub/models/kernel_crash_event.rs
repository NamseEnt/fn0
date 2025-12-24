use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the kernel crash event. A kernel crash event occurs when the kernel detects an exception and triggers a reboot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelCrashEvent {
    pub data: KernelEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for KernelCrashEvent
pub struct KernelCrashEventRequired {
    pub data: KernelEventData,

    pub r#type: String,
}

impl KernelCrashEvent {
    /// Create a new KernelCrashEvent with required fields
    pub fn new(required: KernelCrashEventRequired) -> Self {
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
