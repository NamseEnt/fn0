use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeCapacityTopologyRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute capacity topology.
    pub compute_capacity_topology_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for GetComputeCapacityTopologyRequest
pub struct GetComputeCapacityTopologyRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute capacity topology.
    pub compute_capacity_topology_id: String,
}

impl GetComputeCapacityTopologyRequest {
    /// Create a new GetComputeCapacityTopologyRequest with required fields
    pub fn new(required: GetComputeCapacityTopologyRequestRequired) -> Self {
        Self {
            compute_capacity_topology_id: required.compute_capacity_topology_id,

            opc_request_id: None,
        }
    }

    /// Set compute_capacity_topology_id
    pub fn set_compute_capacity_topology_id(mut self, value: String) -> Self {
        self.compute_capacity_topology_id = value;
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
