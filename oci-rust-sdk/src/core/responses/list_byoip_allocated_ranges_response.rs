use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListByoipAllocatedRangesResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ByoipAllocatedRangeCollection instance.
    pub byoip_allocated_range_collection: ByoipAllocatedRangeCollection,
}


/// Required fields for ListByoipAllocatedRangesResponse
pub struct ListByoipAllocatedRangesResponseRequired {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ByoipAllocatedRangeCollection instance.
    pub byoip_allocated_range_collection: ByoipAllocatedRangeCollection,
}

impl ListByoipAllocatedRangesResponse {
    /// Create a new ListByoipAllocatedRangesResponse with required fields
    pub fn new(required: ListByoipAllocatedRangesResponseRequired) -> Self {
        Self {
            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            byoip_allocated_range_collection: required.byoip_allocated_range_collection,
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

    /// Set byoip_allocated_range_collection
    pub fn set_byoip_allocated_range_collection(mut self, value: ByoipAllocatedRangeCollection) -> Self {
        self.byoip_allocated_range_collection = value;
        self
    }
}


