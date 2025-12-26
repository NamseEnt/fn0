use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListComputeGpuMemoryClusterInstancesResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGpuMemoryClusterInstanceCollection instance.
    pub compute_gpu_memory_cluster_instance_collection: ComputeGpuMemoryClusterInstanceCollection,
}


/// Required fields for ListComputeGpuMemoryClusterInstancesResponse
pub struct ListComputeGpuMemoryClusterInstancesResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGpuMemoryClusterInstanceCollection instance.
    pub compute_gpu_memory_cluster_instance_collection: ComputeGpuMemoryClusterInstanceCollection,
}

impl ListComputeGpuMemoryClusterInstancesResponse {
    /// Create a new ListComputeGpuMemoryClusterInstancesResponse with required fields
    pub fn new(required: ListComputeGpuMemoryClusterInstancesResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            compute_gpu_memory_cluster_instance_collection: required.compute_gpu_memory_cluster_instance_collection,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
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

    /// Set compute_gpu_memory_cluster_instance_collection
    pub fn set_compute_gpu_memory_cluster_instance_collection(mut self, value: ComputeGpuMemoryClusterInstanceCollection) -> Self {
        self.compute_gpu_memory_cluster_instance_collection = value;
        self
    }
}


