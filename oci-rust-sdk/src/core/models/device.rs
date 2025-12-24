use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Device Path corresponding to the block devices attached to instances having a name and isAvailable flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    /// The device name.
    pub name: String,

    /// The flag denoting whether device is available.
    pub is_available: bool,
}

/// Required fields for Device
pub struct DeviceRequired {
    /// The device name.
    pub name: String,

    /// The flag denoting whether device is available.
    pub is_available: bool,
}

impl Device {
    /// Create a new Device with required fields
    pub fn new(required: DeviceRequired) -> Self {
        Self {
            name: required.name,

            is_available: required.is_available,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set is_available
    pub fn set_is_available(mut self, value: bool) -> Self {
        self.is_available = value;
        self
    }
}
