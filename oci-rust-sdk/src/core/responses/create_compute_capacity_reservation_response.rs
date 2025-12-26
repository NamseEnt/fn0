use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComputeCapacityReservationResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,

    /// Location of the resource.
    pub location: String,

    /// The returned model.ComputeCapacityReservation instance.
    pub compute_capacity_reservation: ComputeCapacityReservation,
}

/// Required fields for CreateComputeCapacityReservationResponse
pub struct CreateComputeCapacityReservationResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,

    /// Location of the resource.
    pub location: String,

    /// The returned model.ComputeCapacityReservation instance.
    pub compute_capacity_reservation: ComputeCapacityReservation,
}

impl CreateComputeCapacityReservationResponse {
    /// Create a new CreateComputeCapacityReservationResponse with required fields
    pub fn new(required: CreateComputeCapacityReservationResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            opc_work_request_id: required.opc_work_request_id,

            location: required.location,

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

    /// Set opc_work_request_id
    pub fn set_opc_work_request_id(mut self, value: String) -> Self {
        self.opc_work_request_id = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: String) -> Self {
        self.location = value;
        self
    }

    /// Set compute_capacity_reservation
    pub fn set_compute_capacity_reservation(mut self, value: ComputeCapacityReservation) -> Self {
        self.compute_capacity_reservation = value;
        self
    }
}
