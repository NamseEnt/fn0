use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDhcpOptionsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the set of DHCP options.
    pub dhcp_id: String,
}


/// Required fields for GetDhcpOptionsRequest
pub struct GetDhcpOptionsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the set of DHCP options.
    pub dhcp_id: String,
}

impl GetDhcpOptionsRequest {
    /// Create a new GetDhcpOptionsRequest with required fields
    pub fn new(required: GetDhcpOptionsRequestRequired) -> Self {
        Self {
            dhcp_id: required.dhcp_id,
}
    }

    /// Set dhcp_id
    pub fn set_dhcp_id(mut self, value: String) -> Self {
        self.dhcp_id = value;
        self
    }
}


