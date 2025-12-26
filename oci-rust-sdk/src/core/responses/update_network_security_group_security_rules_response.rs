use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNetworkSecurityGroupSecurityRulesResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.UpdatedNetworkSecurityGroupSecurityRules instance.
    pub updated_network_security_group_security_rules: UpdatedNetworkSecurityGroupSecurityRules,
}


/// Required fields for UpdateNetworkSecurityGroupSecurityRulesResponse
pub struct UpdateNetworkSecurityGroupSecurityRulesResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.UpdatedNetworkSecurityGroupSecurityRules instance.
    pub updated_network_security_group_security_rules: UpdatedNetworkSecurityGroupSecurityRules,
}

impl UpdateNetworkSecurityGroupSecurityRulesResponse {
    /// Create a new UpdateNetworkSecurityGroupSecurityRulesResponse with required fields
    pub fn new(required: UpdateNetworkSecurityGroupSecurityRulesResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            updated_network_security_group_security_rules: required.updated_network_security_group_security_rules,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set updated_network_security_group_security_rules
    pub fn set_updated_network_security_group_security_rules(mut self, value: UpdatedNetworkSecurityGroupSecurityRules) -> Self {
        self.updated_network_security_group_security_rules = value;
        self
    }
}


