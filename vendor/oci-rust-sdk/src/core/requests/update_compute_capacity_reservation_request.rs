use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComputeCapacityReservationRequest {
    /// The OCID of the compute capacity reservation.
    pub capacity_reservation_id: String,

    /// Update compute capacity reservation details.
    pub update_compute_capacity_reservation_details: UpdateComputeCapacityReservationDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for UpdateComputeCapacityReservationRequest
pub struct UpdateComputeCapacityReservationRequestRequired {
    /// The OCID of the compute capacity reservation.
    pub capacity_reservation_id: String,

    /// Update compute capacity reservation details.
    pub update_compute_capacity_reservation_details: UpdateComputeCapacityReservationDetails,
}

impl UpdateComputeCapacityReservationRequest {
    /// Create a new UpdateComputeCapacityReservationRequest with required fields
    pub fn new(required: UpdateComputeCapacityReservationRequestRequired) -> Self {
        Self {
            capacity_reservation_id: required.capacity_reservation_id,

            update_compute_capacity_reservation_details: required
                .update_compute_capacity_reservation_details,

            if_match: None,

            opc_request_id: None,
        }
    }

    /// Set capacity_reservation_id
    pub fn set_capacity_reservation_id(mut self, value: String) -> Self {
        self.capacity_reservation_id = value;
        self
    }

    /// Set update_compute_capacity_reservation_details
    pub fn set_update_compute_capacity_reservation_details(
        mut self,
        value: UpdateComputeCapacityReservationDetails,
    ) -> Self {
        self.update_compute_capacity_reservation_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}
