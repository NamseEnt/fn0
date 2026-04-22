use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The data to create a report of available Compute capacity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComputeCapacityReportDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment. This should always be the root compartment.
    pub compartment_id: String,

    /// The availability domain for the capacity report. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// Information about the shapes in the capacity report.
    pub shape_availabilities: Vec<CreateCapacityReportShapeAvailabilityDetails>,
}

/// Required fields for CreateComputeCapacityReportDetails
pub struct CreateComputeCapacityReportDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment. This should always be the root compartment.
    pub compartment_id: String,

    /// The availability domain for the capacity report. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// Information about the shapes in the capacity report.
    pub shape_availabilities: Vec<CreateCapacityReportShapeAvailabilityDetails>,
}

impl CreateComputeCapacityReportDetails {
    /// Create a new CreateComputeCapacityReportDetails with required fields
    pub fn new(required: CreateComputeCapacityReportDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            availability_domain: required.availability_domain,

            shape_availabilities: required.shape_availabilities,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set shape_availabilities
    pub fn set_shape_availabilities(
        mut self,
        value: Vec<CreateCapacityReportShapeAvailabilityDetails>,
    ) -> Self {
        self.shape_availabilities = value;
        self
    }
}
