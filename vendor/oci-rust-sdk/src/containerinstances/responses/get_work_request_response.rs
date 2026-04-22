use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWorkRequestResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// A decimal number representing the number of seconds the client should wait before polling this endpoint again.
    pub retry_after: i64,

    /// The returned model.WorkRequest instance.
    pub work_request: WorkRequest,
}


/// Required fields for GetWorkRequestResponse
pub struct GetWorkRequestResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// A decimal number representing the number of seconds the client should wait before polling this endpoint again.
    pub retry_after: i64,

    /// The returned model.WorkRequest instance.
    pub work_request: WorkRequest,
}

impl GetWorkRequestResponse {
    /// Create a new GetWorkRequestResponse with required fields
    pub fn new(required: GetWorkRequestResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            retry_after: required.retry_after,

            work_request: required.work_request,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set retry_after
    pub fn set_retry_after(mut self, value: i64) -> Self {
        self.retry_after = value;
        self
    }

    /// Set work_request
    pub fn set_work_request(mut self, value: WorkRequest) -> Self {
        self.work_request = value;
        self
    }
}


