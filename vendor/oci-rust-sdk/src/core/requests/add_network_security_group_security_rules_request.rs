use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddNetworkSecurityGroupSecurityRulesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,

    /// Request with one or more security rules to be associated with the network security group.
    pub add_network_security_group_security_rules_details:
        AddNetworkSecurityGroupSecurityRulesDetails,
}

/// Required fields for AddNetworkSecurityGroupSecurityRulesRequest
pub struct AddNetworkSecurityGroupSecurityRulesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the network security group.
    pub network_security_group_id: String,

    /// Request with one or more security rules to be associated with the network security group.
    pub add_network_security_group_security_rules_details:
        AddNetworkSecurityGroupSecurityRulesDetails,
}

impl AddNetworkSecurityGroupSecurityRulesRequest {
    /// Create a new AddNetworkSecurityGroupSecurityRulesRequest with required fields
    pub fn new(required: AddNetworkSecurityGroupSecurityRulesRequestRequired) -> Self {
        Self {
            network_security_group_id: required.network_security_group_id,

            add_network_security_group_security_rules_details: required
                .add_network_security_group_security_rules_details,
        }
    }

    /// Set network_security_group_id
    pub fn set_network_security_group_id(mut self, value: String) -> Self {
        self.network_security_group_id = value;
        self
    }

    /// Set add_network_security_group_security_rules_details
    pub fn set_add_network_security_group_security_rules_details(
        mut self,
        value: AddNetworkSecurityGroupSecurityRulesDetails,
    ) -> Self {
        self.add_network_security_group_security_rules_details = value;
        self
    }
}
