use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVcnRequest {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    pub vcn_id: String,
}


/// Required fields for GetVcnRequest
pub struct GetVcnRequestRequired {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    pub vcn_id: String,
}

impl GetVcnRequest {
    /// Create a new GetVcnRequest with required fields
    pub fn new(required: GetVcnRequestRequired) -> Self {
        Self {
            vcn_id: required.vcn_id,
}
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
        self
    }
}


