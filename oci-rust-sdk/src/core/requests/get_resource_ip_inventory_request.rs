use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceIpInventoryRequest {
    /// Specify the ID of the resource.
    pub data_request_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetResourceIpInventoryRequest
pub struct GetResourceIpInventoryRequestRequired {
    /// Specify the ID of the resource.
    pub data_request_id: String,
}

impl GetResourceIpInventoryRequest {
    /// Create a new GetResourceIpInventoryRequest with required fields
    pub fn new(required: GetResourceIpInventoryRequestRequired) -> Self {
        Self {
            data_request_id: required.data_request_id,

            opc_request_id: None,
}
    }

    /// Set data_request_id
    pub fn set_data_request_id(mut self, value: String) -> Self {
        self.data_request_id = value;
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


