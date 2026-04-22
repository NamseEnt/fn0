use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListComputeCapacityTopologyComputeBareMetalHostsResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeBareMetalHostCollection instance.
    pub compute_bare_metal_host_collection: ComputeBareMetalHostCollection,
}

/// Required fields for ListComputeCapacityTopologyComputeBareMetalHostsResponse
pub struct ListComputeCapacityTopologyComputeBareMetalHostsResponseRequired {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeBareMetalHostCollection instance.
    pub compute_bare_metal_host_collection: ComputeBareMetalHostCollection,
}

impl ListComputeCapacityTopologyComputeBareMetalHostsResponse {
    /// Create a new ListComputeCapacityTopologyComputeBareMetalHostsResponse with required fields
    pub fn new(required: ListComputeCapacityTopologyComputeBareMetalHostsResponseRequired) -> Self {
        Self {
            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            compute_bare_metal_host_collection: required.compute_bare_metal_host_collection,
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

    /// Set compute_bare_metal_host_collection
    pub fn set_compute_bare_metal_host_collection(
        mut self,
        value: ComputeBareMetalHostCollection,
    ) -> Self {
        self.compute_bare_metal_host_collection = value;
        self
    }
}
