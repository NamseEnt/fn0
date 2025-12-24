use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the other remote peering connection (RPC).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRemotePeeringConnectionsDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the RPC you want to peer with.
    pub peer_id: String,

    /// The name of the region that contains the RPC you want to peer with. <p> Example: {@code us-ashburn-1}
    pub peer_region_name: String,
}

/// Required fields for ConnectRemotePeeringConnectionsDetails
pub struct ConnectRemotePeeringConnectionsDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the RPC you want to peer with.
    pub peer_id: String,

    /// The name of the region that contains the RPC you want to peer with. <p> Example: {@code us-ashburn-1}
    pub peer_region_name: String,
}

impl ConnectRemotePeeringConnectionsDetails {
    /// Create a new ConnectRemotePeeringConnectionsDetails with required fields
    pub fn new(required: ConnectRemotePeeringConnectionsDetailsRequired) -> Self {
        Self {
            peer_id: required.peer_id,

            peer_region_name: required.peer_region_name,
        }
    }

    /// Set peer_id
    pub fn set_peer_id(mut self, value: String) -> Self {
        self.peer_id = value;
        self
    }

    /// Set peer_region_name
    pub fn set_peer_region_name(mut self, value: String) -> Self {
        self.peer_region_name = value;
        self
    }
}
