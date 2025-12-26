use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCrossConnectStatusResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CrossConnectStatus instance.
    pub cross_connect_status: CrossConnectStatus,
}


/// Required fields for GetCrossConnectStatusResponse
pub struct GetCrossConnectStatusResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CrossConnectStatus instance.
    pub cross_connect_status: CrossConnectStatus,
}

impl GetCrossConnectStatusResponse {
    /// Create a new GetCrossConnectStatusResponse with required fields
    pub fn new(required: GetCrossConnectStatusResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            cross_connect_status: required.cross_connect_status,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set cross_connect_status
    pub fn set_cross_connect_status(mut self, value: CrossConnectStatus) -> Self {
        self.cross_connect_status = value;
        self
    }
}


