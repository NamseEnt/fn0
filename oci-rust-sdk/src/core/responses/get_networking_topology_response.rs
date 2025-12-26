use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNetworkingTopologyResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.NetworkingTopology instance.
    pub networking_topology: NetworkingTopology,
}

/// Required fields for GetNetworkingTopologyResponse
pub struct GetNetworkingTopologyResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.NetworkingTopology instance.
    pub networking_topology: NetworkingTopology,
}

impl GetNetworkingTopologyResponse {
    /// Create a new GetNetworkingTopologyResponse with required fields
    pub fn new(required: GetNetworkingTopologyResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            networking_topology: required.networking_topology,
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

    /// Set networking_topology
    pub fn set_networking_topology(mut self, value: NetworkingTopology) -> Self {
        self.networking_topology = value;
        self
    }
}
