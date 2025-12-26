use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartContainerInstanceResponse {
    /// Unique Oracle-assigned identifier for the asynchronous request. You can use this to query status of the asynchronous operation.
    pub opc_work_request_id: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}


/// Required fields for StartContainerInstanceResponse
pub struct StartContainerInstanceResponseRequired {
    /// Unique Oracle-assigned identifier for the asynchronous request. You can use this to query status of the asynchronous operation.
    pub opc_work_request_id: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}

impl StartContainerInstanceResponse {
    /// Create a new StartContainerInstanceResponse with required fields
    pub fn new(required: StartContainerInstanceResponseRequired) -> Self {
        Self {
            opc_work_request_id: required.opc_work_request_id,

            opc_request_id: required.opc_request_id,
}
    }

    /// Set opc_work_request_id
    pub fn set_opc_work_request_id(mut self, value: String) -> Self {
        self.opc_work_request_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }
}


