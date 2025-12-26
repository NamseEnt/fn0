use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeGpuMemoryClusterRequest {
    /// The OCID of the compute GPU memory cluster.
    pub compute_gpu_memory_cluster_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetComputeGpuMemoryClusterRequest
pub struct GetComputeGpuMemoryClusterRequestRequired {
    /// The OCID of the compute GPU memory cluster.
    pub compute_gpu_memory_cluster_id: String,
}

impl GetComputeGpuMemoryClusterRequest {
    /// Create a new GetComputeGpuMemoryClusterRequest with required fields
    pub fn new(required: GetComputeGpuMemoryClusterRequestRequired) -> Self {
        Self {
            compute_gpu_memory_cluster_id: required.compute_gpu_memory_cluster_id,

            opc_request_id: None,
}
    }

    /// Set compute_gpu_memory_cluster_id
    pub fn set_compute_gpu_memory_cluster_id(mut self, value: String) -> Self {
        self.compute_gpu_memory_cluster_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


