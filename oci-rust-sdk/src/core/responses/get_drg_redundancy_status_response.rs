use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDrgRedundancyStatusResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DrgRedundancyStatus instance.
    pub drg_redundancy_status: DrgRedundancyStatus,
}


/// Required fields for GetDrgRedundancyStatusResponse
pub struct GetDrgRedundancyStatusResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DrgRedundancyStatus instance.
    pub drg_redundancy_status: DrgRedundancyStatus,
}

impl GetDrgRedundancyStatusResponse {
    /// Create a new GetDrgRedundancyStatusResponse with required fields
    pub fn new(required: GetDrgRedundancyStatusResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            drg_redundancy_status: required.drg_redundancy_status,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set drg_redundancy_status
    pub fn set_drg_redundancy_status(mut self, value: DrgRedundancyStatus) -> Self {
        self.drg_redundancy_status = value;
        self
    }
}


