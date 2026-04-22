use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRemotePeeringConnectionsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the remote peering connection (RPC).
    pub remote_peering_connection_id: String,

    /// Details to connect peering connection with peering connection from remote region
    pub connect_remote_peering_connections_details: ConnectRemotePeeringConnectionsDetails,
}

/// Required fields for ConnectRemotePeeringConnectionsRequest
pub struct ConnectRemotePeeringConnectionsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the remote peering connection (RPC).
    pub remote_peering_connection_id: String,

    /// Details to connect peering connection with peering connection from remote region
    pub connect_remote_peering_connections_details: ConnectRemotePeeringConnectionsDetails,
}

impl ConnectRemotePeeringConnectionsRequest {
    /// Create a new ConnectRemotePeeringConnectionsRequest with required fields
    pub fn new(required: ConnectRemotePeeringConnectionsRequestRequired) -> Self {
        Self {
            remote_peering_connection_id: required.remote_peering_connection_id,

            connect_remote_peering_connections_details: required
                .connect_remote_peering_connections_details,
        }
    }

    /// Set remote_peering_connection_id
    pub fn set_remote_peering_connection_id(mut self, value: String) -> Self {
        self.remote_peering_connection_id = value;
        self
    }

    /// Set connect_remote_peering_connections_details
    pub fn set_connect_remote_peering_connections_details(
        mut self,
        value: ConnectRemotePeeringConnectionsDetails,
    ) -> Self {
        self.connect_remote_peering_connections_details = value;
        self
    }
}
