use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNetworkSecurityGroupRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,
}

/// Required fields for GetNetworkSecurityGroupRequest
pub struct GetNetworkSecurityGroupRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,
}

impl GetNetworkSecurityGroupRequest {
    /// Create a new GetNetworkSecurityGroupRequest with required fields
    pub fn new(required: GetNetworkSecurityGroupRequestRequired) -> Self {
        Self {
            network_security_group_id: required.network_security_group_id,
        }
    }

    /// Set network_security_group_id
    pub fn set_network_security_group_id(mut self, value: String) -> Self {
        self.network_security_group_id = value;
        self
    }
}
