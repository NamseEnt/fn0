use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeCapacityTopologyResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeCapacityTopology instance.
    pub compute_capacity_topology: ComputeCapacityTopology,
}


/// Required fields for GetComputeCapacityTopologyResponse
pub struct GetComputeCapacityTopologyResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeCapacityTopology instance.
    pub compute_capacity_topology: ComputeCapacityTopology,
}

impl GetComputeCapacityTopologyResponse {
    /// Create a new GetComputeCapacityTopologyResponse with required fields
    pub fn new(required: GetComputeCapacityTopologyResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_capacity_topology: required.compute_capacity_topology,
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

    /// Set compute_capacity_topology
    pub fn set_compute_capacity_topology(mut self, value: ComputeCapacityTopology) -> Self {
        self.compute_capacity_topology = value;
        self
    }
}


