use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIpsecCpeDeviceConfigContentResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned stream.Readable | ReadableStream instance.
    pub value: String,
}

/// Required fields for GetIpsecCpeDeviceConfigContentResponse
pub struct GetIpsecCpeDeviceConfigContentResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned stream.Readable | ReadableStream instance.
    pub value: String,
}

impl GetIpsecCpeDeviceConfigContentResponse {
    /// Create a new GetIpsecCpeDeviceConfigContentResponse with required fields
    pub fn new(required: GetIpsecCpeDeviceConfigContentResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            value: required.value,
        }
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set value
    pub fn set_value(mut self, value: String) -> Self {
        self.value = value;
        self
    }
}
