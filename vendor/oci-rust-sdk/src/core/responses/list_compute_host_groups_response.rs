use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListComputeHostGroupsResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeHostGroupCollection instance.
    pub compute_host_group_collection: ComputeHostGroupCollection,
}

/// Required fields for ListComputeHostGroupsResponse
pub struct ListComputeHostGroupsResponseRequired {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeHostGroupCollection instance.
    pub compute_host_group_collection: ComputeHostGroupCollection,
}

impl ListComputeHostGroupsResponse {
    /// Create a new ListComputeHostGroupsResponse with required fields
    pub fn new(required: ListComputeHostGroupsResponseRequired) -> Self {
        Self {
            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            compute_host_group_collection: required.compute_host_group_collection,
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

    /// Set compute_host_group_collection
    pub fn set_compute_host_group_collection(mut self, value: ComputeHostGroupCollection) -> Self {
        self.compute_host_group_collection = value;
        self
    }
}
