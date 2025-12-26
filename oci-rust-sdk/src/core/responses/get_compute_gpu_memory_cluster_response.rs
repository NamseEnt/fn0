use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeGpuMemoryClusterResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGpuMemoryCluster instance.
    pub compute_gpu_memory_cluster: ComputeGpuMemoryCluster,
}


/// Required fields for GetComputeGpuMemoryClusterResponse
pub struct GetComputeGpuMemoryClusterResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGpuMemoryCluster instance.
    pub compute_gpu_memory_cluster: ComputeGpuMemoryCluster,
}

impl GetComputeGpuMemoryClusterResponse {
    /// Create a new GetComputeGpuMemoryClusterResponse with required fields
    pub fn new(required: GetComputeGpuMemoryClusterResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_gpu_memory_cluster: required.compute_gpu_memory_cluster,
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

    /// Set compute_gpu_memory_cluster
    pub fn set_compute_gpu_memory_cluster(mut self, value: ComputeGpuMemoryCluster) -> Self {
        self.compute_gpu_memory_cluster = value;
        self
    }
}


