use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubnetTopologyResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.SubnetTopology instance.
    pub subnet_topology: SubnetTopology,
}

/// Required fields for GetSubnetTopologyResponse
pub struct GetSubnetTopologyResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.SubnetTopology instance.
    pub subnet_topology: SubnetTopology,
}

impl GetSubnetTopologyResponse {
    /// Create a new GetSubnetTopologyResponse with required fields
    pub fn new(required: GetSubnetTopologyResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            subnet_topology: required.subnet_topology,
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

    /// Set subnet_topology
    pub fn set_subnet_topology(mut self, value: SubnetTopology) -> Self {
        self.subnet_topology = value;
        self
    }
}
