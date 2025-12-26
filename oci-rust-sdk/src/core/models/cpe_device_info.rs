use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Basic information about a particular CPE device type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpeDeviceInfo {
    /// The vendor that makes the CPE device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    /// The platform or software version of the CPE device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_software_version: Option<String>,
}

impl CpeDeviceInfo {
    /// Create a new CpeDeviceInfo
    pub fn new() -> Self {
        Self {
            vendor: None,

            platform_software_version: None,
        }
    }

    /// Set vendor
    pub fn set_vendor(mut self, value: Option<String>) -> Self {
        self.vendor = value;
        self
    }

    /// Set platform_software_version
    pub fn set_platform_software_version(mut self, value: Option<String>) -> Self {
        self.platform_software_version = value;
        self
    }

    /// Set vendor (unwraps Option)
    pub fn with_vendor(mut self, value: impl Into<String>) -> Self {
        self.vendor = Some(value.into());
        self
    }

    /// Set platform_software_version (unwraps Option)
    pub fn with_platform_software_version(mut self, value: impl Into<String>) -> Self {
        self.platform_software_version = Some(value.into());
        self
    }
}

impl Default for CpeDeviceInfo {
    fn default() -> Self {
        Self::new()
    }
}
