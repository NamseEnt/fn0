use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A report of the host capacity within an availability domain that is available for you to create compute instances. Host capacity is the physical infrastructure that resources such as compute instances run on. <p> Use the capacity report to determine whether sufficient capacity is available for a shape before you create an instance or change the shape of an instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeCapacityReport {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment. This should always be the root compartment.
    pub compartment_id: String,

    /// The availability domain for the capacity report. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// Information about the available capacity for each shape in a capacity report.
    pub shape_availabilities: Vec<CapacityReportShapeAvailability>,

    /// The date and time the capacity report was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

/// Required fields for ComputeCapacityReport
pub struct ComputeCapacityReportRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment. This should always be the root compartment.
    pub compartment_id: String,

    /// The availability domain for the capacity report. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// Information about the available capacity for each shape in a capacity report.
    pub shape_availabilities: Vec<CapacityReportShapeAvailability>,

    /// The date and time the capacity report was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl ComputeCapacityReport {
    /// Create a new ComputeCapacityReport with required fields
    pub fn new(required: ComputeCapacityReportRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            availability_domain: required.availability_domain,

            shape_availabilities: required.shape_availabilities,

            time_created: required.time_created,
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
    pub fn set_shape_availabilities(mut self, value: Vec<CapacityReportShapeAvailability>) -> Self {
        self.shape_availabilities = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }
}
