use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddNetworkSecurityGroupSecurityRulesResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.AddedNetworkSecurityGroupSecurityRules instance.
    pub added_network_security_group_security_rules: AddedNetworkSecurityGroupSecurityRules,
}


/// Required fields for AddNetworkSecurityGroupSecurityRulesResponse
pub struct AddNetworkSecurityGroupSecurityRulesResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.AddedNetworkSecurityGroupSecurityRules instance.
    pub added_network_security_group_security_rules: AddedNetworkSecurityGroupSecurityRules,
}

impl AddNetworkSecurityGroupSecurityRulesResponse {
    /// Create a new AddNetworkSecurityGroupSecurityRulesResponse with required fields
    pub fn new(required: AddNetworkSecurityGroupSecurityRulesResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            added_network_security_group_security_rules: required.added_network_security_group_security_rules,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set added_network_security_group_security_rules
    pub fn set_added_network_security_group_security_rules(mut self, value: AddedNetworkSecurityGroupSecurityRules) -> Self {
        self.added_network_security_group_security_rules = value;
        self
    }
}


