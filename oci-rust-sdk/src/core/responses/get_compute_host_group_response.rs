use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeHostGroupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeHostGroup instance.
    pub compute_host_group: ComputeHostGroup,
}

/// Required fields for GetComputeHostGroupResponse
pub struct GetComputeHostGroupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeHostGroup instance.
    pub compute_host_group: ComputeHostGroup,
}

impl GetComputeHostGroupResponse {
    /// Create a new GetComputeHostGroupResponse with required fields
    pub fn new(required: GetComputeHostGroupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_host_group: required.compute_host_group,
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

    /// Set compute_host_group
    pub fn set_compute_host_group(mut self, value: ComputeHostGroup) -> Self {
        self.compute_host_group = value;
        self
    }
}
