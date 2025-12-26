use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveNetworkSecurityGroupSecurityRulesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,

    /// Request with one or more security rules associated with the network security group that will be removed.
    pub remove_network_security_group_security_rules_details: RemoveNetworkSecurityGroupSecurityRulesDetails,
}


/// Required fields for RemoveNetworkSecurityGroupSecurityRulesRequest
pub struct RemoveNetworkSecurityGroupSecurityRulesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,

    /// Request with one or more security rules associated with the network security group that will be removed.
    pub remove_network_security_group_security_rules_details: RemoveNetworkSecurityGroupSecurityRulesDetails,
}

impl RemoveNetworkSecurityGroupSecurityRulesRequest {
    /// Create a new RemoveNetworkSecurityGroupSecurityRulesRequest with required fields
    pub fn new(required: RemoveNetworkSecurityGroupSecurityRulesRequestRequired) -> Self {
        Self {
            network_security_group_id: required.network_security_group_id,

            remove_network_security_group_security_rules_details: required.remove_network_security_group_security_rules_details,
}
    }

    /// Set network_security_group_id
    pub fn set_network_security_group_id(mut self, value: String) -> Self {
        self.network_security_group_id = value;
        self
    }

    /// Set remove_network_security_group_security_rules_details
    pub fn set_remove_network_security_group_security_rules_details(mut self, value: RemoveNetworkSecurityGroupSecurityRulesDetails) -> Self {
        self.remove_network_security_group_security_rules_details = value;
        self
    }
}


