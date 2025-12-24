use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeByoasnCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the BYOASN resource move.
    pub compartment_id: String,
}

/// Required fields for ChangeByoasnCompartmentDetails
pub struct ChangeByoasnCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the BYOASN resource move.
    pub compartment_id: String,
}

impl ChangeByoasnCompartmentDetails {
    /// Create a new ChangeByoasnCompartmentDetails with required fields
    pub fn new(required: ChangeByoasnCompartmentDetailsRequired) -> Self {
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
