use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The detailed information about a particular CPE device type. Compare with {@link CpeDeviceShapeSummary}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpeDeviceShapeDetail {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE device shape. This value uniquely identifies the type of CPE device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_device_shape_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_device_info: Option<CpeDeviceInfo>,

    /// For certain CPE devices types, the customer can provide answers to questions that are specific to the device type. This attribute contains a list of those questions. The Networking service merges the answers with other information and renders a set of CPE configuration content. To provide the answers, use {@link #updateTunnelCpeDeviceConfig(UpdateTunnelCpeDeviceConfigRequest) updateTunnelCpeDeviceConfig}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<CpeDeviceConfigQuestion>>,

    /// A template of CPE device configuration information that will be merged with the customer's answers to the questions to render the final CPE device configuration content. Also see: <p> {@link #getCpeDeviceConfigContent(GetCpeDeviceConfigContentRequest) getCpeDeviceConfigContent} * {@link #getIpsecCpeDeviceConfigContent(GetIpsecCpeDeviceConfigContentRequest) getIpsecCpeDeviceConfigContent} * {@link #getTunnelCpeDeviceConfigContent(GetTunnelCpeDeviceConfigContentRequest) getTunnelCpeDeviceConfigContent}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

impl CpeDeviceShapeDetail {
    /// Create a new CpeDeviceShapeDetail
    pub fn new() -> Self {
        Self {
            cpe_device_shape_id: None,

            cpe_device_info: None,

            parameters: None,

            template: None,
        }
    }

    /// Set cpe_device_shape_id
    pub fn set_cpe_device_shape_id(mut self, value: Option<String>) -> Self {
        self.cpe_device_shape_id = value;
        self
    }

    /// Set cpe_device_info
    pub fn set_cpe_device_info(mut self, value: Option<CpeDeviceInfo>) -> Self {
        self.cpe_device_info = value;
        self
    }

    /// Set parameters
    pub fn set_parameters(mut self, value: Option<Vec<CpeDeviceConfigQuestion>>) -> Self {
        self.parameters = value;
        self
    }

    /// Set template
    pub fn set_template(mut self, value: Option<String>) -> Self {
        self.template = value;
        self
    }

    /// Set cpe_device_shape_id (unwraps Option)
    pub fn with_cpe_device_shape_id(mut self, value: impl Into<String>) -> Self {
        self.cpe_device_shape_id = Some(value.into());
        self
    }

    /// Set cpe_device_info (unwraps Option)
    pub fn with_cpe_device_info(mut self, value: CpeDeviceInfo) -> Self {
        self.cpe_device_info = Some(value);
        self
    }

    /// Set parameters (unwraps Option)
    pub fn with_parameters(mut self, value: Vec<CpeDeviceConfigQuestion>) -> Self {
        self.parameters = Some(value);
        self
    }

    /// Set template (unwraps Option)
    pub fn with_template(mut self, value: impl Into<String>) -> Self {
        self.template = Some(value.into());
        self
    }
}

impl Default for CpeDeviceShapeDetail {
    fn default() -> Self {
        Self::new()
    }
}
