use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCpeDeviceShapeResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CpeDeviceShapeDetail instance.
    pub cpe_device_shape_detail: CpeDeviceShapeDetail,
}


/// Required fields for GetCpeDeviceShapeResponse
pub struct GetCpeDeviceShapeResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CpeDeviceShapeDetail instance.
    pub cpe_device_shape_detail: CpeDeviceShapeDetail,
}

impl GetCpeDeviceShapeResponse {
    /// Create a new GetCpeDeviceShapeResponse with required fields
    pub fn new(required: GetCpeDeviceShapeResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            cpe_device_shape_detail: required.cpe_device_shape_detail,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set cpe_device_shape_detail
    pub fn set_cpe_device_shape_detail(mut self, value: CpeDeviceShapeDetail) -> Self {
        self.cpe_device_shape_detail = value;
        self
    }
}


