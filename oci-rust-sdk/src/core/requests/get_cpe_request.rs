use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCpeRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE.
    pub cpe_id: String,
}

/// Required fields for GetCpeRequest
pub struct GetCpeRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE.
    pub cpe_id: String,
}

impl GetCpeRequest {
    /// Create a new GetCpeRequest with required fields
    pub fn new(required: GetCpeRequestRequired) -> Self {
        Self {
            cpe_id: required.cpe_id,
        }
    }

    /// Set cpe_id
    pub fn set_cpe_id(mut self, value: String) -> Self {
        self.cpe_id = value;
        self
    }
}
