use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListByoasnsResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ByoasnCollection instance.
    pub byoasn_collection: ByoasnCollection,
}


/// Required fields for ListByoasnsResponse
pub struct ListByoasnsResponseRequired {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ByoasnCollection instance.
    pub byoasn_collection: ByoasnCollection,
}

impl ListByoasnsResponse {
    /// Create a new ListByoasnsResponse with required fields
    pub fn new(required: ListByoasnsResponseRequired) -> Self {
        Self {
            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            byoasn_collection: required.byoasn_collection,
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

    /// Set byoasn_collection
    pub fn set_byoasn_collection(mut self, value: ByoasnCollection) -> Self {
        self.byoasn_collection = value;
        self
    }
}


