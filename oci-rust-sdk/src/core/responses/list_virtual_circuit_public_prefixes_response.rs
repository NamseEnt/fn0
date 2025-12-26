use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVirtualCircuitPublicPrefixesResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// A list of VirtualCircuitPublicPrefix instances.
    pub items: Vec<VirtualCircuitPublicPrefix>,
}


/// Required fields for ListVirtualCircuitPublicPrefixesResponse
pub struct ListVirtualCircuitPublicPrefixesResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// A list of VirtualCircuitPublicPrefix instances.
    pub items: Vec<VirtualCircuitPublicPrefix>,
}

impl ListVirtualCircuitPublicPrefixesResponse {
    /// Create a new ListVirtualCircuitPublicPrefixesResponse with required fields
    pub fn new(required: ListVirtualCircuitPublicPrefixesResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            items: required.items,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<VirtualCircuitPublicPrefix>) -> Self {
        self.items = value;
        self
    }
}


