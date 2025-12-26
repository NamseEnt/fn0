use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeGpuMemoryFabricResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGpuMemoryFabric instance.
    pub compute_gpu_memory_fabric: ComputeGpuMemoryFabric,
}


/// Required fields for GetComputeGpuMemoryFabricResponse
pub struct GetComputeGpuMemoryFabricResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGpuMemoryFabric instance.
    pub compute_gpu_memory_fabric: ComputeGpuMemoryFabric,
}

impl GetComputeGpuMemoryFabricResponse {
    /// Create a new GetComputeGpuMemoryFabricResponse with required fields
    pub fn new(required: GetComputeGpuMemoryFabricResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_gpu_memory_fabric: required.compute_gpu_memory_fabric,
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

    /// Set compute_gpu_memory_fabric
    pub fn set_compute_gpu_memory_fabric(mut self, value: ComputeGpuMemoryFabric) -> Self {
        self.compute_gpu_memory_fabric = value;
        self
    }
}


