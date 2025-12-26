use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComputeClusterResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeCluster instance.
    pub compute_cluster: ComputeCluster,
}

/// Required fields for CreateComputeClusterResponse
pub struct CreateComputeClusterResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeCluster instance.
    pub compute_cluster: ComputeCluster,
}

impl CreateComputeClusterResponse {
    /// Create a new CreateComputeClusterResponse with required fields
    pub fn new(required: CreateComputeClusterResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_cluster: required.compute_cluster,
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

    /// Set compute_cluster
    pub fn set_compute_cluster(mut self, value: ComputeCluster) -> Self {
        self.compute_cluster = value;
        self
    }
}
