use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVlanRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VLAN.
    pub vlan_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetVlanRequest
pub struct GetVlanRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VLAN.
    pub vlan_id: String,
}

impl GetVlanRequest {
    /// Create a new GetVlanRequest with required fields
    pub fn new(required: GetVlanRequestRequired) -> Self {
        Self {
            vlan_id: required.vlan_id,

            opc_request_id: None,
}
    }

    /// Set vlan_id
    pub fn set_vlan_id(mut self, value: String) -> Self {
        self.vlan_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


