use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVnicRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    pub vnic_id: String,
}


/// Required fields for GetVnicRequest
pub struct GetVnicRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    pub vnic_id: String,
}

impl GetVnicRequest {
    /// Create a new GetVnicRequest with required fields
    pub fn new(required: GetVnicRequestRequired) -> Self {
        Self {
            vnic_id: required.vnic_id,
}
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: String) -> Self {
        self.vnic_id = value;
        self
    }
}


