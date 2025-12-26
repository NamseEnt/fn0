use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of CPE configuration answers for the tunnel, which the customer provides in {@link #updateTunnelCpeDeviceConfig(UpdateTunnelCpeDeviceConfigRequest) updateTunnelCpeDeviceConfig}. The answers correlate to the questions that are specific to the CPE device type (see the {@code parameters} attribute of {@link CpeDeviceShapeDetail}). <p> See these related operations: <p> {@link #getTunnelCpeDeviceConfig(GetTunnelCpeDeviceConfigRequest) getTunnelCpeDeviceConfig} * {@link #getTunnelCpeDeviceConfigContent(GetTunnelCpeDeviceConfigContentRequest) getTunnelCpeDeviceConfigContent} * {@link #getIpsecCpeDeviceConfigContent(GetIpsecCpeDeviceConfigContentRequest) getIpsecCpeDeviceConfigContent} * {@link #getCpeDeviceConfigContent(GetCpeDeviceConfigContentRequest) getCpeDeviceConfigContent}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelCpeDeviceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_cpe_device_config_parameter: Option<Vec<CpeDeviceConfigAnswer>>,
}

impl TunnelCpeDeviceConfig {
    /// Create a new TunnelCpeDeviceConfig
    pub fn new() -> Self {
        Self {
            tunnel_cpe_device_config_parameter: None,
        }
    }

    /// Set tunnel_cpe_device_config_parameter
    pub fn set_tunnel_cpe_device_config_parameter(
        mut self,
        value: Option<Vec<CpeDeviceConfigAnswer>>,
    ) -> Self {
        self.tunnel_cpe_device_config_parameter = value;
        self
    }

    /// Set tunnel_cpe_device_config_parameter (unwraps Option)
    pub fn with_tunnel_cpe_device_config_parameter(
        mut self,
        value: Vec<CpeDeviceConfigAnswer>,
    ) -> Self {
        self.tunnel_cpe_device_config_parameter = Some(value);
        self
    }
}

impl Default for TunnelCpeDeviceConfig {
    fn default() -> Self {
        Self::new()
    }
}
