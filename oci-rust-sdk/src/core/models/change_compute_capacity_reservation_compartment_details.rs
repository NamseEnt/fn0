use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the compartment to move the compute capacity reservation to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeComputeCapacityReservationCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute capacity reservation to.
    pub compartment_id: String,
}

/// Required fields for ChangeComputeCapacityReservationCompartmentDetails
pub struct ChangeComputeCapacityReservationCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute capacity reservation to.
    pub compartment_id: String,
}

impl ChangeComputeCapacityReservationCompartmentDetails {
    /// Create a new ChangeComputeCapacityReservationCompartmentDetails with required fields
    pub fn new(required: ChangeComputeCapacityReservationCompartmentDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }
}
