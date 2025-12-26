use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIPSecConnectionTunnelSecurityAssociationsResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// For list pagination. A pagination token to get the total number of results available.
    pub opc_total_items: i64,

    /// A list of TunnelSecurityAssociationSummary instances.
    pub items: Vec<TunnelSecurityAssociationSummary>,
}


/// Required fields for ListIPSecConnectionTunnelSecurityAssociationsResponse
pub struct ListIPSecConnectionTunnelSecurityAssociationsResponseRequired {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// For list pagination. A pagination token to get the total number of results available.
    pub opc_total_items: i64,

    /// A list of TunnelSecurityAssociationSummary instances.
    pub items: Vec<TunnelSecurityAssociationSummary>,
}

impl ListIPSecConnectionTunnelSecurityAssociationsResponse {
    /// Create a new ListIPSecConnectionTunnelSecurityAssociationsResponse with required fields
    pub fn new(required: ListIPSecConnectionTunnelSecurityAssociationsResponseRequired) -> Self {
        Self {
            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            opc_total_items: required.opc_total_items,

            items: required.items,
}
    }

    /// Set opc_next_page
    pub fn set_opc_next_page(mut self, value: String) -> Self {
        self.opc_next_page = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_total_items
    pub fn set_opc_total_items(mut self, value: i64) -> Self {
        self.opc_total_items = value;
        self
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<TunnelSecurityAssociationSummary>) -> Self {
        self.items = value;
        self
    }
}


