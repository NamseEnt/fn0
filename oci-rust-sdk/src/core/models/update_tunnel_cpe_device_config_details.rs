use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTunnelCpeDeviceConfigDetails {
    /// The set of configuration answers for a CPE device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_cpe_device_config: Option<Vec<CpeDeviceConfigAnswer>>,
}

impl UpdateTunnelCpeDeviceConfigDetails {
    /// Create a new UpdateTunnelCpeDeviceConfigDetails
    pub fn new() -> Self {
        Self {
            tunnel_cpe_device_config: None,
        }
    }

    /// Set tunnel_cpe_device_config
    pub fn set_tunnel_cpe_device_config(
        mut self,
        value: Option<Vec<CpeDeviceConfigAnswer>>,
    ) -> Self {
        self.tunnel_cpe_device_config = value;
        self
    }

    /// Set tunnel_cpe_device_config (unwraps Option)
    pub fn with_tunnel_cpe_device_config(mut self, value: Vec<CpeDeviceConfigAnswer>) -> Self {
        self.tunnel_cpe_device_config = Some(value);
        self
    }
}

impl Default for UpdateTunnelCpeDeviceConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
