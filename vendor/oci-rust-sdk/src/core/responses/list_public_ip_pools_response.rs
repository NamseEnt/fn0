use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPublicIpPoolsResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.PublicIpPoolCollection instance.
    pub public_ip_pool_collection: PublicIpPoolCollection,
}

/// Required fields for ListPublicIpPoolsResponse
pub struct ListPublicIpPoolsResponseRequired {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.PublicIpPoolCollection instance.
    pub public_ip_pool_collection: PublicIpPoolCollection,
}

impl ListPublicIpPoolsResponse {
    /// Create a new ListPublicIpPoolsResponse with required fields
    pub fn new(required: ListPublicIpPoolsResponseRequired) -> Self {
        Self {
            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            public_ip_pool_collection: required.public_ip_pool_collection,
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

    /// Set public_ip_pool_collection
    pub fn set_public_ip_pool_collection(mut self, value: PublicIpPoolCollection) -> Self {
        self.public_ip_pool_collection = value;
        self
    }
}
