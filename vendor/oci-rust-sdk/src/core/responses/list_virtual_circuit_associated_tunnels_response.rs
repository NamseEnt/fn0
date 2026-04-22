use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVirtualCircuitAssociatedTunnelsResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// A list of VirtualCircuitAssociatedTunnelDetails instances.
    pub items: Vec<VirtualCircuitAssociatedTunnelDetails>,
}

/// Required fields for ListVirtualCircuitAssociatedTunnelsResponse
pub struct ListVirtualCircuitAssociatedTunnelsResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// A list of VirtualCircuitAssociatedTunnelDetails instances.
    pub items: Vec<VirtualCircuitAssociatedTunnelDetails>,
}

impl ListVirtualCircuitAssociatedTunnelsResponse {
    /// Create a new ListVirtualCircuitAssociatedTunnelsResponse with required fields
    pub fn new(required: ListVirtualCircuitAssociatedTunnelsResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            opc_next_page: required.opc_next_page,

            items: required.items,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_next_page
    pub fn set_opc_next_page(mut self, value: String) -> Self {
        self.opc_next_page = value;
        self
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<VirtualCircuitAssociatedTunnelDetails>) -> Self {
        self.items = value;
        self
    }
}
