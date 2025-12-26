use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIPSecConnectionDeviceConfigResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnectionDeviceConfig instance.
    pub i_psec_connection_device_config: IPSecConnectionDeviceConfig,
}


/// Required fields for GetIPSecConnectionDeviceConfigResponse
pub struct GetIPSecConnectionDeviceConfigResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnectionDeviceConfig instance.
    pub i_psec_connection_device_config: IPSecConnectionDeviceConfig,
}

impl GetIPSecConnectionDeviceConfigResponse {
    /// Create a new GetIPSecConnectionDeviceConfigResponse with required fields
    pub fn new(required: GetIPSecConnectionDeviceConfigResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            i_psec_connection_device_config: required.i_psec_connection_device_config,
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

    /// Set i_psec_connection_device_config
    pub fn set_i_psec_connection_device_config(mut self, value: IPSecConnectionDeviceConfig) -> Self {
        self.i_psec_connection_device_config = value;
        self
    }
}


