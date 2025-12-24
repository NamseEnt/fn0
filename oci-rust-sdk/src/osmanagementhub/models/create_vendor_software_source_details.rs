use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to replicated a vendor software source into another compartment (other than root).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVendorSoftwareSourceDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the vendor software source in the root compartment that is being replicated.
    pub origin_software_source_id: String,

    pub software_source_type: String,
}

/// Required fields for CreateVendorSoftwareSourceDetails
pub struct CreateVendorSoftwareSourceDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the vendor software source in the root compartment that is being replicated.
    pub origin_software_source_id: String,

    pub software_source_type: String,
}

impl CreateVendorSoftwareSourceDetails {
    /// Create a new CreateVendorSoftwareSourceDetails with required fields
    pub fn new(required: CreateVendorSoftwareSourceDetailsRequired) -> Self {
        Self {
            origin_software_source_id: required.origin_software_source_id,

            software_source_type: required.software_source_type,
        }
    }

    /// Set origin_software_source_id
    pub fn set_origin_software_source_id(mut self, value: String) -> Self {
        self.origin_software_source_id = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }
}
