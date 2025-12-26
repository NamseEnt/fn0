use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// These configuration details are used in the move operation when changing the compartment containing a capture filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeCaptureFilterCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the VTAP capture filter move.
    pub compartment_id: String,
}

/// Required fields for ChangeCaptureFilterCompartmentDetails
pub struct ChangeCaptureFilterCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the VTAP capture filter move.
    pub compartment_id: String,
}

impl ChangeCaptureFilterCompartmentDetails {
    /// Create a new ChangeCaptureFilterCompartmentDetails with required fields
    pub fn new(required: ChangeCaptureFilterCompartmentDetailsRequired) -> Self {
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
