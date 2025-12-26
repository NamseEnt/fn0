use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNetworkSecurityGroupSecurityRulesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,

    /// Request with one or more security rules associated with the network security group that will be updated.
    pub update_network_security_group_security_rules_details: UpdateNetworkSecurityGroupSecurityRulesDetails,
}


/// Required fields for UpdateNetworkSecurityGroupSecurityRulesRequest
pub struct UpdateNetworkSecurityGroupSecurityRulesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,

    /// Request with one or more security rules associated with the network security group that will be updated.
    pub update_network_security_group_security_rules_details: UpdateNetworkSecurityGroupSecurityRulesDetails,
}

impl UpdateNetworkSecurityGroupSecurityRulesRequest {
    /// Create a new UpdateNetworkSecurityGroupSecurityRulesRequest with required fields
    pub fn new(required: UpdateNetworkSecurityGroupSecurityRulesRequestRequired) -> Self {
        Self {
            network_security_group_id: required.network_security_group_id,

            update_network_security_group_security_rules_details: required.update_network_security_group_security_rules_details,
}
    }

    /// Set network_security_group_id
    pub fn set_network_security_group_id(mut self, value: String) -> Self {
        self.network_security_group_id = value;
        self
    }

    /// Set update_network_security_group_security_rules_details
    pub fn set_update_network_security_group_security_rules_details(mut self, value: UpdateNetworkSecurityGroupSecurityRulesDetails) -> Self {
        self.update_network_security_group_security_rules_details = value;
        self
    }
}


