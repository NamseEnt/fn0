use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWorkRequestRequest {
    /// The ID of the asynchronous request.
    pub work_request_id: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetWorkRequestRequest
pub struct GetWorkRequestRequestRequired {
    /// The ID of the asynchronous request.
    pub work_request_id: String,
}

impl GetWorkRequestRequest {
    /// Create a new GetWorkRequestRequest with required fields
    pub fn new(required: GetWorkRequestRequestRequired) -> Self {
        Self {
            work_request_id: required.work_request_id,

            opc_request_id: None,
}
    }

    /// Set work_request_id
    pub fn set_work_request_id(mut self, value: String) -> Self {
        self.work_request_id = value;
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


