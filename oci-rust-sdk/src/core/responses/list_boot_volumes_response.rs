use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBootVolumesResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// A list of BootVolume instances.
    pub items: Vec<BootVolume>,
}


/// Required fields for ListBootVolumesResponse
pub struct ListBootVolumesResponseRequired {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// A list of BootVolume instances.
    pub items: Vec<BootVolume>,
}

impl ListBootVolumesResponse {
    /// Create a new ListBootVolumesResponse with required fields
    pub fn new(required: ListBootVolumesResponseRequired) -> Self {
        Self {
            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

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

    /// Set items
    pub fn set_items(mut self, value: Vec<BootVolume>) -> Self {
        self.items = value;
        self
    }
}


