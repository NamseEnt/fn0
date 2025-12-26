use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTunnelCpeDeviceConfigResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.TunnelCpeDeviceConfig instance.
    pub tunnel_cpe_device_config: TunnelCpeDeviceConfig,
}


/// Required fields for UpdateTunnelCpeDeviceConfigResponse
pub struct UpdateTunnelCpeDeviceConfigResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.TunnelCpeDeviceConfig instance.
    pub tunnel_cpe_device_config: TunnelCpeDeviceConfig,
}

impl UpdateTunnelCpeDeviceConfigResponse {
    /// Create a new UpdateTunnelCpeDeviceConfigResponse with required fields
    pub fn new(required: UpdateTunnelCpeDeviceConfigResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            tunnel_cpe_device_config: required.tunnel_cpe_device_config,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set tunnel_cpe_device_config
    pub fn set_tunnel_cpe_device_config(mut self, value: TunnelCpeDeviceConfig) -> Self {
        self.tunnel_cpe_device_config = value;
        self
    }
}


