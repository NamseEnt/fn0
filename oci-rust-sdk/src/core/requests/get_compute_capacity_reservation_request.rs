use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeCapacityReservationRequest {
    /// The OCID of the compute capacity reservation.
    pub capacity_reservation_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetComputeCapacityReservationRequest
pub struct GetComputeCapacityReservationRequestRequired {
    /// The OCID of the compute capacity reservation.
    pub capacity_reservation_id: String,
}

impl GetComputeCapacityReservationRequest {
    /// Create a new GetComputeCapacityReservationRequest with required fields
    pub fn new(required: GetComputeCapacityReservationRequestRequired) -> Self {
        Self {
            capacity_reservation_id: required.capacity_reservation_id,

            opc_request_id: None,
}
    }

    /// Set capacity_reservation_id
    pub fn set_capacity_reservation_id(mut self, value: String) -> Self {
        self.capacity_reservation_id = value;
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


