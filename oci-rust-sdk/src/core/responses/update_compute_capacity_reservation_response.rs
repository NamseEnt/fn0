use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComputeCapacityReservationResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,
}


/// Required fields for UpdateComputeCapacityReservationResponse
pub struct UpdateComputeCapacityReservationResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,
}

impl UpdateComputeCapacityReservationResponse {
    /// Create a new UpdateComputeCapacityReservationResponse with required fields
    pub fn new(required: UpdateComputeCapacityReservationResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            opc_work_request_id: required.opc_work_request_id,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_work_request_id
    pub fn set_opc_work_request_id(mut self, value: String) -> Self {
        self.opc_work_request_id = value;
        self
    }
}


