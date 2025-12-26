use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIPSecConnectionDeviceConfigRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,
}

/// Required fields for GetIPSecConnectionDeviceConfigRequest
pub struct GetIPSecConnectionDeviceConfigRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,
}

impl GetIPSecConnectionDeviceConfigRequest {
    /// Create a new GetIPSecConnectionDeviceConfigRequest with required fields
    pub fn new(required: GetIPSecConnectionDeviceConfigRequestRequired) -> Self {
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
