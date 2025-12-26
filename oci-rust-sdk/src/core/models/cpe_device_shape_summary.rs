use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A summary of information about a particular CPE device type. Compare with {@link CpeDeviceShapeDetail}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpeDeviceShapeSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE device shape. This value uniquely identifies the type of CPE device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_device_info: Option<CpeDeviceInfo>,
}

impl CpeDeviceShapeSummary {
    /// Create a new CpeDeviceShapeSummary
    pub fn new() -> Self {
        Self {
            id: None,

            cpe_device_info: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set cpe_device_info
    pub fn set_cpe_device_info(mut self, value: Option<CpeDeviceInfo>) -> Self {
        self.cpe_device_info = value;
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set cpe_device_info (unwraps Option)
    pub fn with_cpe_device_info(mut self, value: CpeDeviceInfo) -> Self {
        self.cpe_device_info = Some(value);
        self
    }
}

impl Default for CpeDeviceShapeSummary {
    fn default() -> Self {
        Self::new()
    }
}
