use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCpeDeviceShapeRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE device shape.
    pub cpe_device_shape_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetCpeDeviceShapeRequest
pub struct GetCpeDeviceShapeRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE device shape.
    pub cpe_device_shape_id: String,
}

impl GetCpeDeviceShapeRequest {
    /// Create a new GetCpeDeviceShapeRequest with required fields
    pub fn new(required: GetCpeDeviceShapeRequestRequired) -> Self {
        Self {
            cpe_device_shape_id: required.cpe_device_shape_id,

            opc_request_id: None,
}
    }

    /// Set cpe_device_shape_id
    pub fn set_cpe_device_shape_id(mut self, value: String) -> Self {
        self.cpe_device_shape_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


