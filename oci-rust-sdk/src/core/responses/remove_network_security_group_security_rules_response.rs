use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveNetworkSecurityGroupSecurityRulesResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}


/// Required fields for RemoveNetworkSecurityGroupSecurityRulesResponse
pub struct RemoveNetworkSecurityGroupSecurityRulesResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}

impl RemoveNetworkSecurityGroupSecurityRulesResponse {
    /// Create a new RemoveNetworkSecurityGroupSecurityRulesResponse with required fields
    pub fn new(required: RemoveNetworkSecurityGroupSecurityRulesResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }
}


