use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that defines the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) and the availability of a vendor software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSourceAvailability {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the vendor software source.
    pub software_source_id: String,

    /// Availability of the software source to instances in private data centers or third-party clouds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<Availability>,

    /// Availability of the software source to OCI instances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_at_oci: Option<Availability>,
}

/// Required fields for SoftwareSourceAvailability
pub struct SoftwareSourceAvailabilityRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the vendor software source.
    pub software_source_id: String,
}

impl SoftwareSourceAvailability {
    /// Create a new SoftwareSourceAvailability with required fields
    pub fn new(required: SoftwareSourceAvailabilityRequired) -> Self {
        Self {
            software_source_id: required.software_source_id,

            availability: None,

            availability_at_oci: None,
        }
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: String) -> Self {
        self.software_source_id = value;
        self
    }

    /// Set availability
    pub fn set_availability(mut self, value: Option<Availability>) -> Self {
        self.availability = value;
        self
    }

    /// Set availability_at_oci
    pub fn set_availability_at_oci(mut self, value: Option<Availability>) -> Self {
        self.availability_at_oci = value;
        self
    }

    /// Set availability (unwraps Option)
    pub fn with_availability(mut self, value: Availability) -> Self {
        self.availability = Some(value);
        self
    }

    /// Set availability_at_oci (unwraps Option)
    pub fn with_availability_at_oci(mut self, value: Availability) -> Self {
        self.availability_at_oci = Some(value);
        self
    }
}
