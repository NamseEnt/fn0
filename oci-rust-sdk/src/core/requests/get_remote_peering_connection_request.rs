use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRemotePeeringConnectionRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the remote peering connection (RPC).
    pub remote_peering_connection_id: String,
}


/// Required fields for GetRemotePeeringConnectionRequest
pub struct GetRemotePeeringConnectionRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the remote peering connection (RPC).
    pub remote_peering_connection_id: String,
}

impl GetRemotePeeringConnectionRequest {
    /// Create a new GetRemotePeeringConnectionRequest with required fields
    pub fn new(required: GetRemotePeeringConnectionRequestRequired) -> Self {
        Self {
            remote_peering_connection_id: required.remote_peering_connection_id,
}
    }

    /// Set remote_peering_connection_id
    pub fn set_remote_peering_connection_id(mut self, value: String) -> Self {
        self.remote_peering_connection_id = value;
        self
    }
}


