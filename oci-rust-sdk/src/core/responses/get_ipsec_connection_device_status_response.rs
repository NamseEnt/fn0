use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIPSecConnectionDeviceStatusResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnectionDeviceStatus instance.
    pub i_psec_connection_device_status: IPSecConnectionDeviceStatus,
}

/// Required fields for GetIPSecConnectionDeviceStatusResponse
pub struct GetIPSecConnectionDeviceStatusResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnectionDeviceStatus instance.
    pub i_psec_connection_device_status: IPSecConnectionDeviceStatus,
}

impl GetIPSecConnectionDeviceStatusResponse {
    /// Create a new GetIPSecConnectionDeviceStatusResponse with required fields
    pub fn new(required: GetIPSecConnectionDeviceStatusResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            i_psec_connection_device_status: required.i_psec_connection_device_status,
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

    /// Set i_psec_connection_device_status
    pub fn set_i_psec_connection_device_status(
        mut self,
        value: IPSecConnectionDeviceStatus,
    ) -> Self {
        self.i_psec_connection_device_status = value;
        self
    }
}
