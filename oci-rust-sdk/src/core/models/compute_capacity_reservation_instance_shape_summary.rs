use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An available shape used to launch instances in a compute capacity reservation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeCapacityReservationInstanceShapeSummary {
    /// The shape's availability domain.
    pub availability_domain: String,

    /// The name of the available shape used to launch instances in a compute capacity reservation.
    pub instance_shape: String,
}

/// Required fields for ComputeCapacityReservationInstanceShapeSummary
pub struct ComputeCapacityReservationInstanceShapeSummaryRequired {
    /// The shape's availability domain.
    pub availability_domain: String,

    /// The name of the available shape used to launch instances in a compute capacity reservation.
    pub instance_shape: String,
}

impl ComputeCapacityReservationInstanceShapeSummary {
    /// Create a new ComputeCapacityReservationInstanceShapeSummary with required fields
    pub fn new(required: ComputeCapacityReservationInstanceShapeSummaryRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            instance_shape: required.instance_shape,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set instance_shape
    pub fn set_instance_shape(mut self, value: String) -> Self {
        self.instance_shape = value;
        self
    }
}
