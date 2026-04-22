use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeCapacityReservationResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeCapacityReservation instance.
    pub compute_capacity_reservation: ComputeCapacityReservation,
}

/// Required fields for GetComputeCapacityReservationResponse
pub struct GetComputeCapacityReservationResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeCapacityReservation instance.
    pub compute_capacity_reservation: ComputeCapacityReservation,
}

impl GetComputeCapacityReservationResponse {
    /// Create a new GetComputeCapacityReservationResponse with required fields
    pub fn new(required: GetComputeCapacityReservationResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_capacity_reservation: required.compute_capacity_reservation,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set compute_capacity_reservation
    pub fn set_compute_capacity_reservation(mut self, value: ComputeCapacityReservation) -> Self {
        self.compute_capacity_reservation = value;
        self
    }
}
