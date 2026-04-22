use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIPSecConnectionRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,
}

/// Required fields for GetIPSecConnectionRequest
pub struct GetIPSecConnectionRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,
}

impl GetIPSecConnectionRequest {
    /// Create a new GetIPSecConnectionRequest with required fields
    pub fn new(required: GetIPSecConnectionRequestRequired) -> Self {
        Self {
            ipsc_id: required.ipsc_id,
        }
    }

    /// Set ipsc_id
    pub fn set_ipsc_id(mut self, value: String) -> Self {
        self.ipsc_id = value;
        self
    }
}
