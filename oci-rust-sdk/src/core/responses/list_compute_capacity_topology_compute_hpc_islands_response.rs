use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListComputeCapacityTopologyComputeHpcIslandsResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeHpcIslandCollection instance.
    pub compute_hpc_island_collection: ComputeHpcIslandCollection,
}

/// Required fields for ListComputeCapacityTopologyComputeHpcIslandsResponse
pub struct ListComputeCapacityTopologyComputeHpcIslandsResponseRequired {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeHpcIslandCollection instance.
    pub compute_hpc_island_collection: ComputeHpcIslandCollection,
}

impl ListComputeCapacityTopologyComputeHpcIslandsResponse {
    /// Create a new ListComputeCapacityTopologyComputeHpcIslandsResponse with required fields
    pub fn new(required: ListComputeCapacityTopologyComputeHpcIslandsResponseRequired) -> Self {
        Self {
            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            compute_hpc_island_collection: required.compute_hpc_island_collection,
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

    /// Set compute_hpc_island_collection
    pub fn set_compute_hpc_island_collection(mut self, value: ComputeHpcIslandCollection) -> Self {
        self.compute_hpc_island_collection = value;
        self
    }
}
